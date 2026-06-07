use roxmltree::Document;

#[derive(Debug, Clone)]
pub struct SvgRectRef {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone)]
pub struct SvgCircleRef {
    pub id: String,
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
}

#[derive(Debug, Clone)]
pub struct SvgScene {
    pub width: f32,
    pub height: f32,
    pub rooms: Vec<SvgRectRef>,
    pub blocks: Vec<SvgRectRef>,
    pub slots: Vec<SvgCircleRef>,
}

impl SvgScene {
    pub fn parse(svg: &str) -> Result<Self, String> {
        let doc = Document::parse(svg).map_err(|e| format!("SVG parse failed: {e}"))?;
        let root = doc
            .descendants()
            .find(|n| n.is_element() && n.tag_name().name() == "svg")
            .ok_or_else(|| "SVG root element not found".to_string())?;

        let (width, height) = parse_viewbox_or_size(&root)?;

        let mut rooms = Vec::new();
        let mut blocks = Vec::new();
        let mut slots = Vec::new();

        for node in root.descendants().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                "rect" => {
                    let Some(id) = node.attribute("id") else {
                        continue;
                    };
                    let x = parse_f32(node.attribute("x"));
                    let y = parse_f32(node.attribute("y"));
                    let w = parse_f32(node.attribute("width"));
                    let h = parse_f32(node.attribute("height"));
                    let rect = SvgRectRef {
                        id: id.to_string(),
                        x,
                        y,
                        w,
                        h,
                    };
                    if id.starts_with("room_") {
                        rooms.push(rect);
                    } else {
                        blocks.push(rect);
                    }
                }
                "circle" => {
                    let Some(id) = node.attribute("id") else {
                        continue;
                    };
                    if !id.starts_with("slot_") {
                        continue;
                    }
                    slots.push(SvgCircleRef {
                        id: id.to_string(),
                        cx: parse_f32(node.attribute("cx")),
                        cy: parse_f32(node.attribute("cy")),
                        r: parse_f32(node.attribute("r")),
                    });
                }
                _ => {}
            }
        }

        Ok(Self {
            width,
            height,
            rooms,
            blocks,
            slots,
        })
    }
}

fn parse_viewbox_or_size(root: &roxmltree::Node) -> Result<(f32, f32), String> {
    if let Some(vb) = root.attribute("viewBox") {
        let parts: Vec<f32> = vb
            .split_ascii_whitespace()
            .filter_map(|s| s.parse::<f32>().ok())
            .collect();
        if parts.len() == 4 {
            return Ok((parts[2], parts[3]));
        }
    }

    let width = parse_f32(root.attribute("width"));
    let height = parse_f32(root.attribute("height"));
    if width > 0.0 && height > 0.0 {
        Ok((width, height))
    } else {
        Err("SVG must define viewBox or width/height".to_string())
    }
}

fn parse_f32(value: Option<&str>) -> f32 {
    value
        .unwrap_or("0")
        .replace("px", "")
        .parse::<f32>()
        .unwrap_or(0.0)
}
