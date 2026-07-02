//! Random chapter node-graph generation (Slay-the-Spire-style layered DAG).
//!
//! Layers run bottom (entry) to top (boss). Each node links to 1–2 nodes in
//! the next layer; links never cross, and every node is reachable.

use super::super::core::Rng;
use super::ChapterDef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Fight,  // 遭遇
    Elite,  // 强袭
    Event,  // 奇遇
    Story,  // 情缘 / 剧情
    Rest,   // 歇脚
    Market, // 集市
    Boss,   // 首领
}

impl NodeKind {
    /// One-glyph CJK marker rendered on the node (unifont-safe).
    pub fn glyph(self) -> &'static str {
        match self {
            NodeKind::Fight => "战",
            NodeKind::Elite => "袭",
            NodeKind::Event => "遇",
            NodeKind::Story => "缘",
            NodeKind::Rest => "歇",
            NodeKind::Market => "市",
            NodeKind::Boss => "魔",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            NodeKind::Fight => "遭遇战",
            NodeKind::Elite => "精英强袭",
            NodeKind::Event => "奇遇",
            NodeKind::Story => "剧情",
            NodeKind::Rest => "歇脚",
            NodeKind::Market => "集市",
            NodeKind::Boss => "章末首领",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub layer: usize,
    /// Horizontal slot within the layer (for drawing).
    pub slot: usize,
    /// Slots in this node's layer (for centering when drawing).
    pub layer_width: usize,
    /// Indices of reachable nodes in the next layer.
    pub next: Vec<usize>,
    pub cleared: bool,
}

#[derive(Clone, Debug)]
pub struct NodeGraph {
    pub nodes: Vec<Node>,
    pub layer_count: usize,
}

impl NodeGraph {
    /// Layout: entry layer (2–3 nodes) → `depth` random layers (2–3 nodes,
    /// one of which is replaced by a single-node Story layer) → rest layer
    /// (single node) → boss layer (single node).
    pub fn generate(def: &ChapterDef, rng: &mut Rng) -> Self {
        // Decide layer widths & which mid layer is the story layer.
        let mut widths: Vec<usize> = Vec::new();
        widths.push(rng.range(2, 3) as usize); // entry
        let story_layer = rng.range(1, def.depth as i32) as usize;
        for i in 1..=def.depth {
            if i == story_layer {
                widths.push(1); // story bottleneck
            } else {
                widths.push(rng.range(2, 3) as usize);
            }
        }
        widths.push(1); // rest before boss
        widths.push(1); // boss
        let layer_count = widths.len();

        // Create nodes.
        let mut nodes: Vec<Node> = Vec::new();
        let mut layer_start: Vec<usize> = Vec::new();
        for (layer, &w) in widths.iter().enumerate() {
            layer_start.push(nodes.len());
            for slot in 0..w {
                let kind = if layer == layer_count - 1 {
                    NodeKind::Boss
                } else if layer == layer_count - 2 {
                    NodeKind::Rest
                } else if layer == story_layer && layer != 0 {
                    NodeKind::Story
                } else {
                    roll_kind(rng, layer)
                };
                nodes.push(Node {
                    kind,
                    layer,
                    slot,
                    layer_width: w,
                    next: Vec::new(),
                    cleared: false,
                });
            }
        }

        // Link adjacent layers without crossings: node at fraction f of its
        // layer connects to the node(s) around fraction f of the next layer.
        for layer in 0..layer_count - 1 {
            let (a0, aw) = (layer_start[layer], widths[layer]);
            let (b0, bw) = (layer_start[layer + 1], widths[layer + 1]);
            for i in 0..aw {
                let target = if aw == 1 {
                    bw / 2
                } else {
                    (i * (bw - 1) + (aw - 1) / 2) / (aw - 1).max(1)
                };
                let target = target.min(bw - 1);
                let node = a0 + i;
                nodes[node].next.push(b0 + target);
                // Occasionally add a fork to a horizontal neighbour.
                if bw > 1 && rng.chance(0.45) {
                    let alt = if target + 1 < bw {
                        target + 1
                    } else {
                        target - 1
                    };
                    nodes[node].next.push(b0 + alt);
                }
            }
            // Ensure every next-layer node has an incoming edge.
            for j in 0..bw {
                let covered = (0..aw).any(|i| nodes[a0 + i].next.contains(&(b0 + j)));
                if !covered {
                    // Attach to the nearest source slot.
                    let src = a0 + (j * aw.saturating_sub(1)) / bw.max(1).min(aw);
                    let src = src.min(a0 + aw - 1);
                    nodes[src].next.push(b0 + j);
                }
            }
            for i in 0..aw {
                nodes[a0 + i].next.sort_unstable();
                nodes[a0 + i].next.dedup();
            }
        }

        Self { nodes, layer_count }
    }

    pub fn entry_nodes(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.layer == 0)
            .map(|(i, _)| i)
            .collect()
    }
}

/// Node-kind distribution for regular layers. Entry layers avoid elites so a
/// fresh chapter never opens with a spike.
fn roll_kind(rng: &mut Rng, layer: usize) -> NodeKind {
    let roll = rng.unit();
    if layer >= 2 && roll < 0.16 {
        NodeKind::Elite
    } else if roll < 0.46 {
        NodeKind::Fight
    } else if roll < 0.68 {
        NodeKind::Event
    } else if roll < 0.79 {
        NodeKind::Rest
    } else if roll < 0.88 {
        NodeKind::Market
    } else {
        NodeKind::Fight
    }
}
