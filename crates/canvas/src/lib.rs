use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasNodeKind {
    Text { text: String },
    File { file: String, subpath: Option<String> },
    Link { url: String },
    Group { label: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasNode {
    pub id: String,
    pub bounds: Rect,
    pub color: Option<String>,
    pub kind: CanvasNodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl EdgeSide {
    fn parse(s: Option<&str>) -> Option<Self> {
        match s {
            Some("top") => Some(Self::Top),
            Some("right") => Some(Self::Right),
            Some("bottom") => Some(Self::Bottom),
            Some("left") => Some(Self::Left),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasEdge {
    pub id: String,
    pub from_node: String,
    pub from_side: Option<EdgeSide>,
    pub to_node: String,
    pub to_side: Option<EdgeSide>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CanvasDocument {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
}

#[derive(Deserialize)]
struct RawCanvas {
    #[serde(default)]
    nodes: Vec<RawNode>,
    #[serde(default)]
    edges: Vec<RawEdge>,
}

#[derive(Deserialize)]
struct RawNode {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    #[serde(default = "default_width")]
    width: f64,
    #[serde(default = "default_height")]
    height: f64,
    color: Option<String>,
    text: Option<String>,
    file: Option<String>,
    subpath: Option<String>,
    url: Option<String>,
    label: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEdge {
    id: String,
    from_node: String,
    from_side: Option<String>,
    to_node: String,
    to_side: Option<String>,
    label: Option<String>,
}

fn default_width() -> f64 {
    240.0
}

fn default_height() -> f64 {
    120.0
}

impl CanvasDocument {
    pub fn from_json_str(raw: &str) -> Result<Self, String> {
        let parsed: RawCanvas = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        let nodes = parsed
            .nodes
            .into_iter()
            .map(|n| {
                let kind = match n.node_type.as_str() {
                    "file" => CanvasNodeKind::File {
                        file: n.file.unwrap_or_default(),
                        subpath: n.subpath,
                    },
                    "link" => CanvasNodeKind::Link {
                        url: n.url.unwrap_or_default(),
                    },
                    "group" => CanvasNodeKind::Group { label: n.label },
                    _ => CanvasNodeKind::Text {
                        text: n.text.unwrap_or_default(),
                    },
                };
                CanvasNode {
                    id: n.id,
                    bounds: Rect {
                        x: n.x,
                        y: n.y,
                        width: n.width,
                        height: n.height,
                    },
                    color: n.color,
                    kind,
                }
            })
            .collect();

        let edges = parsed
            .edges
            .into_iter()
            .map(|e| CanvasEdge {
                id: e.id,
                from_node: e.from_node,
                from_side: EdgeSide::parse(e.from_side.as_deref()),
                to_node: e.to_node,
                to_side: EdgeSide::parse(e.to_side.as_deref()),
                label: e.label,
            })
            .collect();

        Ok(Self { nodes, edges })
    }

    pub fn add_node(&mut self, node: CanvasNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: CanvasEdge) {
        self.edges.push(edge);
    }

    pub fn node(&self, id: &str) -> Option<&CanvasNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn dangling_edges(&self) -> Vec<&CanvasEdge> {
        self.edges
            .iter()
            .filter(|e| self.node(&e.from_node).is_none() || self.node(&e.to_node).is_none())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_dangling_edges() {
        let mut doc = CanvasDocument::default();
        doc.add_node(CanvasNode {
            id: "n1".into(),
            bounds: Rect { x: 0.0, y: 0.0, width: 240.0, height: 120.0 },
            color: None,
            kind: CanvasNodeKind::Text { text: "Idea".into() },
        });
        doc.add_edge(CanvasEdge {
            id: "e1".into(),
            from_node: "n1".into(),
            from_side: Some(EdgeSide::Right),
            to_node: "n_missing".into(),
            to_side: Some(EdgeSide::Left),
            label: None,
        });
        assert_eq!(doc.dangling_edges().len(), 1);
    }

    #[test]
    fn parses_json_canvas_spec() {
        let raw = r#"{
            "nodes": [
                {"id":"a","type":"text","x":10,"y":20,"width":200,"height":100,"text":"Hello"},
                {"id":"b","type":"file","x":300,"y":20,"width":220,"height":140,"file":"Start.md"}
            ],
            "edges": [
                {"id":"e1","fromNode":"a","fromSide":"right","toNode":"b","toSide":"left","label":"links"}
            ]
        }"#;
        let doc = CanvasDocument::from_json_str(raw).expect("parse canvas");
        assert_eq!(doc.nodes.len(), 2);
        assert_eq!(doc.edges.len(), 1);
        assert_eq!(doc.dangling_edges().len(), 0);
        assert_eq!(doc.edges[0].label.as_deref(), Some("links"));
    }
}
