use rand::Rng;

// ─── OCP Opcodes (miroir de binary/opcodes.rs) ────────────────────────────────
const NODE_TEXT: u8 = 0x01;
const NODE_ELEMENT: u8 = 0x02;
const ATTR_TEXT: u8 = 0x10;
const ATTR_EXPRESSION: u8 = 0x11;
const ATTR_BOOLEAN: u8 = 0x12;
const ATTR_AST: u8 = 0x13;

// ─── Payload mutation ─────────────────────────────────────────────────────────

/// Génère une version mutée d'un payload OCP valide.
/// Simule une corruption réseau ou un attaquant envoyant des paquets forgés.
pub fn mutate_ocp_payload(rng: &mut impl Rng, original: &[u8]) -> Vec<u8> {
    let mut data = original.to_vec();
    if data.is_empty() {
        return data;
    }

    let mutation_count = rng.gen_range(1..=5);

    for _ in 0..mutation_count {
        let index = rng.gen_range(0..data.len());
        match rng.gen_range(0..6) {
            // Bit flip
            0 => {
                let bit = rng.gen_range(0..8u8);
                data[index] ^= 1 << bit;
            }
            // Byte substitution aléatoire
            1 => {
                data[index] = rng.gen();
            }
            // Valeurs limites — critiques pour les headers de longueur
            2 => {
                let boundaries = [0u8, 1, 127, 128, 254, 255];
                data[index] = boundaries[rng.gen_range(0..boundaries.len())];
            }
            // Suppression de bloc → déclenche "Unexpected EOF"
            3 => {
                if data.len() > 1 {
                    data.remove(index);
                }
            }
            // Insertion d'un octet aléatoire → décale tous les offsets
            4 => {
                data.insert(index, rng.gen());
            }
            // Duplication d'un segment → gonfle artificiellement les longueurs
            5 => {
                let end = (index + rng.gen_range(1..=8)).min(data.len());
                let segment = data[index..end].to_vec();
                for (j, byte) in segment.iter().enumerate() {
                    data.insert(end + j, *byte);
                }
            }
            _ => {}
        }
    }

    data
}

// ─── Payload structuré valide ─────────────────────────────────────────────────

/// Encode un entier u32 en little-endian.
fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Encode un entier u16 en little-endian.
fn write_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Encode une chaîne préfixée par sa longueur u16.
fn write_str_u16(buf: &mut Vec<u8>, s: &str) {
    write_u16(buf, s.len() as u16);
    buf.extend_from_slice(s.as_bytes());
}

/// Encode une chaîne préfixée par sa longueur u32.
fn write_str_u32(buf: &mut Vec<u8>, s: &str) {
    write_u32(buf, s.len() as u32);
    buf.extend_from_slice(s.as_bytes());
}

/// Produit un nœud texte OCP valide.
fn encode_text_node(buf: &mut Vec<u8>, content: &str) {
    buf.push(NODE_TEXT);
    write_str_u32(buf, content);
}

/// Produit un nœud élément OCP valide avec attributs et enfants.
fn encode_element_node(buf: &mut Vec<u8>, tag: &str, attrs: &[(&str, &str)], children: &[Vec<u8>]) {
    buf.push(NODE_ELEMENT);
    write_str_u16(buf, tag);
    // self_closing
    buf.push(0);
    // attributs
    write_u16(buf, attrs.len() as u16);
    for (k, v) in attrs {
        write_str_u16(buf, k);
        buf.push(ATTR_TEXT);
        write_str_u16(buf, v);
    }
    // enfants
    let total_children: u32 = children.len() as u32;
    write_u32(buf, total_children);
    for child in children {
        buf.extend_from_slice(child);
    }
}

// ─── Générateurs de payloads de test ─────────────────────────────────────────

/// Payload OCP minimal valide : 0 nœuds racine.
pub fn generate_empty_ocp() -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, 0); // root_count = 0
    buf
}

/// Payload OCP valide avec N nœuds texte à la racine.
pub fn generate_flat_ocp(node_count: u32, content: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, node_count);
    for _ in 0..node_count {
        encode_text_node(&mut buf, content);
    }
    buf
}

/// Payload OCP récursif : éléments imbriqués à `depth` niveaux.
/// Teste les limites de pile du décodeur (stack overflow potentiel).
pub fn generate_deep_ocp(depth: usize) -> Vec<u8> {
    let mut root_buf = Vec::new();
    write_u32(&mut root_buf, 1); // 1 nœud racine

    // Construit l'arbre de l'intérieur vers l'extérieur
    let mut inner: Vec<u8> = Vec::new();
    encode_text_node(&mut inner, "leaf");

    for d in 0..depth {
        let tag = format!("Level{}", d);
        let mut outer = Vec::new();
        encode_element_node(&mut outer, &tag, &[], &[inner.clone()]);
        inner = outer;
    }

    root_buf.extend_from_slice(&inner);
    root_buf
}

/// Payload OCP avec un header root_count=u32::MAX → devrait déclencher une
/// allocation refusée ou une erreur propre, pas un crash ou un OOM.
pub fn generate_overflow_root_count() -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, u32::MAX); // root_count absurde
    // Aucun nœud réel — le décodeur doit gérer l'EOF prématuré proprement
    buf
}

/// Payload OCP avec une longueur de string u32::MAX → EOF prématuré.
pub fn generate_overflow_string_length() -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, 1); // 1 nœud racine
    buf.push(NODE_TEXT);
    write_u32(&mut buf, u32::MAX); // longueur absurde
    // Pas d'octets de string qui suivent
    buf
}

/// Payload OCP avec un opcode inconnu → le décodeur doit retourner Err, pas paniquer.
pub fn generate_unknown_opcode() -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, 1);
    buf.push(0xFF); // opcode inexistant
    buf
}

/// Payload OCP avec un attr_count=u16::MAX → doit être rejeté proprement.
pub fn generate_overflow_attr_count() -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, 1);
    buf.push(NODE_ELEMENT);
    write_str_u16(&mut buf, "Box");
    buf.push(0); // self_closing = false
    write_u16(&mut buf, u16::MAX); // attr_count absurde
    // Aucun attribut réel
    buf
}

/// Génère un payload OCP aléatoire entièrement structuré (valide mais pathologique).
pub fn generate_random_valid_ocp(rng: &mut impl Rng, node_count: u32, max_depth: usize) -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, node_count);
    for _ in 0..node_count {
        let depth = rng.gen_range(0..=max_depth);
        let node = random_node(rng, depth);
        buf.extend_from_slice(&node);
    }
    buf
}

fn random_node(rng: &mut impl Rng, remaining_depth: usize) -> Vec<u8> {
    if remaining_depth == 0 || rng.gen_bool(0.4) {
        // Feuille : nœud texte
        let content: String = (0..rng.gen_range(1..=64))
            .map(|_| rng.gen_range(b'a'..=b'z') as char)
            .collect();
        let mut buf = Vec::new();
        encode_text_node(&mut buf, &content);
        buf
    } else {
        // Nœud élément avec enfants récursifs
        let tag = format!("Tag{}", rng.gen_range(0..10));
        let attr_count = rng.gen_range(0..=3usize);
        let attrs: Vec<(String, String)> = (0..attr_count)
            .map(|_| {
                let k: String = (0..4).map(|_| rng.gen_range(b'a'..=b'z') as char).collect();
                let v: String = (0..8).map(|_| rng.gen_range(b'a'..=b'z') as char).collect();
                (k, v)
            })
            .collect();
        let attr_refs: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

        let child_count = rng.gen_range(0..=3usize);
        let children: Vec<Vec<u8>> = (0..child_count)
            .map(|_| random_node(rng, remaining_depth - 1))
            .collect();

        let mut buf = Vec::new();
        encode_element_node(&mut buf, &tag, &attr_refs, &children);
        buf
    }
}