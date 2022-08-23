use anyhow::{anyhow, bail, Context, Result};
use bevy::math::Vec2;
use std::collections::HashMap;

/// Polyline (straight line segments)
pub struct Line {
    pub id: String,
    pub pos: Vec<Vec2>,
}

/// Circle or ellipse
pub struct Point {
    pub id: String,
    pub pos: Vec2,
    /// For ellipse uses biggest of extents
    pub radius: f32,
}

#[derive(Default)]
pub struct File {
    pub title: Option<String>,
    pub lines: Vec<Line>,
    pub points: Vec<Point>,
}

impl File {
    /// Read data from SVG file.
    /// This is really minimalistic and was tested only with Inkscape SVG.
    ///
    /// If it fails with "invalid float literal" set this in Inkscape:
    /// `Edit -> Preferences -> Input/Output -> SVG Export -> Path data -> Path string format -> Absolute`
    pub fn from_bytes(file: &[u8]) -> Result<File> {
        let xml = load_xml(file)?;

        let mut file = File::default();
        file.parse(&xml)?;

        if file.lines.is_empty() && file.points.is_empty() {
            bail!("File contained empty SVG")
        }
        Ok(file)
    }

    /// Minimal and maximal coordinates
    pub fn minmax(&self) -> (Vec2, Vec2) {
        let (min, max) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));
        let (min, max) = self.lines.iter().fold((min, max), |(min, max), ln| {
            ln.pos
                .iter()
                .fold((min, max), |(min, max), p| (p.min(min), p.max(max)))
        });
        let (min, max) = self.points.iter().fold((min, max), |(min, max), pt| {
            (
                min.min(pt.pos - Vec2::splat(pt.radius)),
                max.max(pt.pos + Vec2::splat(pt.radius)),
            )
        });
        (min, max)
    }

    /// Modifies coordinates to fit into the specified range, keeping proportions
    pub fn _resize(&mut self, min: Vec2, max: Vec2) {
        let (now_min, now_max) = self.minmax();

        let scale = ((max - min) / (now_max - now_min)).min_element();
        let translate = min - now_min * scale;

        self.transform(translate, scale)
    }

    /// Modifies coordinates (of everything)
    pub fn transform(&mut self, translate: Vec2, scale: f32) {
        for ln in &mut self.lines {
            for p in &mut ln.pos {
                *p *= scale;
                *p += translate;
            }
        }
        for pt in &mut self.points {
            pt.pos *= scale;
            pt.pos += translate;
            pt.radius *= scale;
        }
    }

    /// Move to zero and set correct scaling
    pub fn fix(&mut self) {
        let (min, max) = self.minmax();
        let center = (min + max) / 2.;
        self.transform(-center, 1.);

        for ln in &mut self.lines {
            for p in &mut ln.pos {
                p.y = -p.y
            }
        }
        for pt in &mut self.points {
            pt.pos.y = -pt.pos.y
        }
    }
}

impl File {
    fn parse(&mut self, el: &XmlElement) -> Result<()> {
        for e in &el.elements {
            match e.name.as_str() {
                "g" => self.parse(e)?,
                "path" => self.lines.push(Self::parse_path(e).with_context(|| {
                    format!("Path, id: \"{}\"", e.get_attr("id").unwrap_or(""))
                })?),
                "rect" => self.lines.push(Self::parse_rect(e).with_context(|| {
                    format!("Rect, id: \"{}\"", e.get_attr("id").unwrap_or(""))
                })?),
                "circle" | "ellipse" => {
                    self.points.push(Self::parse_circle(e).with_context(|| {
                        format!("Circle, id: \"{}\"", e.get_attr("id").unwrap_or(""))
                    })?)
                }
                "title" => self.title = e.content.clone(),
                _ => (),
            }
        }
        Ok(())
    }

    fn parse_path(e: &XmlElement) -> Result<Line> {
        let mut vs = e.get_attr("d")?.split(&[' ', ','][..]);
        let mut pos = Vec::new();

        let rpos = |s: Option<&str>| -> Result<f32> {
            s.unwrap_or_default()
                .parse::<f32>()
                .with_context(|| format!("{:?}", s))
        };

        let mut cur = Vec2::default();
        let mut cmd = 0;

        while let Some(vsi) = vs.next() {
            match vsi {
                "" => (),
                "L" => cmd = 0,
                "l" => cmd = 1,
                "H" => cmd = 2,
                "h" => cmd = 3,
                "V" => cmd = 4,
                "v" => cmd = 5,
                "Z" | "z" => match pos.first() {
                    Some(p) => {
                        let p = *p;
                        pos.push(p)
                    }
                    None => bail!("Invalid loop command"),
                },
                "M" => {
                    cur.x = rpos(vs.next())?;
                    cur.y = rpos(vs.next())?;
                    pos.push(cur);
                }
                _ => match cmd {
                    0 => {
                        cur.x = rpos(Some(vsi))?;
                        cur.y = rpos(vs.next())?;
                        pos.push(cur)
                    }
                    1 => {
                        cur.x += rpos(Some(vsi))?;
                        cur.y += rpos(vs.next())?;
                        pos.push(cur)
                    }
                    2 => {
                        cur.x = rpos(Some(vsi))?;
                        pos.push(cur)
                    }
                    3 => {
                        cur.x += rpos(Some(vsi))?;
                        pos.push(cur)
                    }
                    4 => {
                        cur.y = rpos(Some(vsi))?;
                        pos.push(cur)
                    }
                    5 => {
                        cur.y += rpos(Some(vsi))?;
                        pos.push(cur)
                    }
                    _ => (),
                },
            }
        }

        if pos.is_empty() {
            bail!("Empty path");
        }

        Ok(Line {
            id: e.get_attr("id")?.to_owned(),
            pos,
        })
    }

    fn parse_rect(e: &XmlElement) -> Result<Line> {
        let x1 = e.get_attr("x")?.parse::<f32>()?;
        let y1 = e.get_attr("y")?.parse::<f32>()?;
        let x2 = e.get_attr("width")?.parse::<f32>()? + x1;
        let y2 = e.get_attr("height")?.parse::<f32>()? + y1;

        Ok(Line {
            id: e.get_attr("id")?.to_owned(),
            pos: vec![
                Vec2::new(x1, y1),
                Vec2::new(x2, y1),
                Vec2::new(x2, y2),
                Vec2::new(x1, y2),
                Vec2::new(x1, y1),
            ],
        })
    }

    fn parse_circle(e: &XmlElement) -> Result<Point> {
        Ok(Point {
            id: e.get_attr("id")?.to_owned(),
            pos: Vec2::new(
                e.get_attr("cx")?.parse::<f32>()?,
                e.get_attr("cy")?.parse::<f32>()?,
            ),
            radius: if e.name == "circle" {
                e.get_attr("r")?.parse::<f32>()?
            } else {
                e.get_attr("rx")?
                    .parse::<f32>()?
                    .max(e.get_attr("ry")?.parse::<f32>()?)
            },
        })
    }
}

struct XmlElement {
    name: String,
    attrs: HashMap<String, String>,
    elements: Vec<XmlElement>,
    content: Option<String>,
}

impl XmlElement {
    fn get_attr(&self, name: &str) -> Result<&str> {
        self.attrs
            .get(name)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("No such attribute: {} in {}", name, self.name))
    }
}

fn load_xml(file: &[u8]) -> Result<XmlElement> {
    use xmltree::Element;

    fn conv(src: &Element) -> XmlElement {
        XmlElement {
            name: src.name.clone(),
            attrs: src.attributes.clone(),
            elements: src
                .children
                .iter()
                .filter_map(|n| n.as_element().map(|e| conv(e)))
                .collect(),
            content: src
                .children
                .iter()
                .find_map(|n| n.as_text())
                .map(ToString::to_string),
        }
    }

    Ok(conv(&Element::parse(file)?))
}
