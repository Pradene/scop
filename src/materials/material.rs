use std::collections::HashMap;
use std::io::BufRead;

use lineal::Vector;

#[derive(Debug, PartialEq)]
pub enum MtlLine {
    Comment(String),
    NewMtl(String),
    Ka(f32, f32, f32), // ambient color
    Kd(f32, f32, f32), // diffuse color
    Ks(f32, f32, f32), // specular color
    Ns(f32),           // specular exponent
    Ni(f32),           // optical density
    Dissolve(f32),     // transparency (dissolve)
    Illum(i32),        // illumination model
    MapKa(String),     // ambient texture map
    MapKd(String),     // diffuse texture map
    MapKs(String),     // specular texture map
}

#[derive(Debug, Default, Clone)]
pub struct Material {
    pub name: String,
    pub ka: Option<Vector<f32, 3>>,
    pub kd: Option<Vector<f32, 3>>,
    pub ks: Option<Vector<f32, 3>>,
    pub ns: Option<f32>,
    pub ni: Option<f32>,
    pub dissolve: Option<f32>,
    pub illum: Option<i32>,
    pub map_ka: Option<String>,
    pub map_kd: Option<String>,
    pub map_ks: Option<String>,
}

/// A parser for MTL files that reads from any type implementing BufRead.
pub struct MaterialParser<R> {
    reader: R,
}

impl<R> MaterialParser<R>
where
    R: BufRead,
{
    /// Create a new MaterialParser.
    pub fn new(reader: R) -> MaterialParser<R> {
        MaterialParser { reader }
    }

    /// Parse the entire MTL file into a Material struct with proper grouping.
    pub fn parse(&mut self) -> Result<HashMap<String, Material>, String> {
        let mut materials: HashMap<String, Material> = HashMap::new();
        let mut current = Material::default();

        for line_result in (&mut self.reader).lines() {
            match line_result {
                Ok(line) => {
                    if let Some(parsed) = Self::parse_line(line) {
                        match parsed {
                            MtlLine::NewMtl(name) => {
                                if !current.name.is_empty() {
                                    materials.insert(current.name.clone(), current.clone());
                                }

                                current = Material::default();
                                current.name = name;
                            }
                            MtlLine::Ka(r, g, b) => {
                                current.ka = Some(Vector::new([r, g, b]));
                            }
                            MtlLine::Kd(r, g, b) => {
                                current.kd = Some(Vector::new([r, g, b]));
                            }
                            MtlLine::Ks(r, g, b) => {
                                current.ks = Some(Vector::new([r, g, b]));
                            }
                            MtlLine::Ns(val) => {
                                current.ns = Some(val);
                            }
                            MtlLine::Ni(val) => {
                                current.ni = Some(val);
                            }
                            MtlLine::Dissolve(val) => {
                                current.dissolve = Some(val);
                            }
                            MtlLine::Illum(val) => {
                                current.illum = Some(val);
                            }
                            MtlLine::MapKa(fname) => {
                                current.map_ka = Some(fname);
                            }
                            MtlLine::MapKd(fname) => {
                                current.map_kd = Some(fname);
                            }
                            MtlLine::MapKs(fname) => {
                                current.map_ks = Some(fname);
                            }
                            MtlLine::Comment(_) => {}
                        }
                    }
                }
                Err(e) => return Err(format!("Error reading file: {}", e)),
            }
        }

        if !current.name.is_empty() {
            materials.insert(current.name.clone(), current);
        }

        Ok(materials)
    }

    fn parse_line(line: String) -> Option<MtlLine> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        if trimmed.starts_with('#') {
            return Some(MtlLine::Comment(trimmed[1..].trim().to_string()));
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.is_empty() {
            return None;
        }

        match tokens[0] {
            "newmtl" => {
                if tokens.len() >= 2 {
                    Some(MtlLine::NewMtl(tokens[1..].join(" ")))
                } else {
                    None
                }
            }
            "Ka" => {
                if tokens.len() >= 4 {
                    let r = tokens[1].parse::<f32>().ok()?;
                    let g = tokens[2].parse::<f32>().ok()?;
                    let b = tokens[3].parse::<f32>().ok()?;
                    Some(MtlLine::Ka(r, g, b))
                } else {
                    None
                }
            }
            "Kd" => {
                if tokens.len() >= 4 {
                    let r = tokens[1].parse::<f32>().ok()?;
                    let g = tokens[2].parse::<f32>().ok()?;
                    let b = tokens[3].parse::<f32>().ok()?;
                    Some(MtlLine::Kd(r, g, b))
                } else {
                    None
                }
            }
            "Ks" => {
                if tokens.len() >= 4 {
                    let r = tokens[1].parse::<f32>().ok()?;
                    let g = tokens[2].parse::<f32>().ok()?;
                    let b = tokens[3].parse::<f32>().ok()?;
                    Some(MtlLine::Ks(r, g, b))
                } else {
                    None
                }
            }
            "Ns" => {
                if tokens.len() >= 2 {
                    let value = tokens[1].parse::<f32>().ok()?;
                    Some(MtlLine::Ns(value))
                } else {
                    None
                }
            }
            "Ni" => {
                if tokens.len() >= 2 {
                    let value = tokens[1].parse::<f32>().ok()?;
                    Some(MtlLine::Ni(value))
                } else {
                    None
                }
            }
            "d" => {
                if tokens.len() >= 2 {
                    let value = tokens[1].parse::<f32>().ok()?;
                    Some(MtlLine::Dissolve(value))
                } else {
                    None
                }
            }
            "illum" => {
                if tokens.len() >= 2 {
                    let value = tokens[1].parse::<i32>().ok()?;
                    Some(MtlLine::Illum(value))
                } else {
                    None
                }
            }
            "map_Ka" => {
                if tokens.len() >= 2 {
                    Some(MtlLine::MapKa(tokens[1..].join(" ")))
                } else {
                    None
                }
            }
            "map_Kd" => {
                if tokens.len() >= 2 {
                    Some(MtlLine::MapKd(tokens[1..].join(" ")))
                } else {
                    None
                }
            }
            "map_Ks" => {
                if tokens.len() >= 2 {
                    Some(MtlLine::MapKs(tokens[1..].join(" ")))
                } else {
                    None
                }
            }
            "#" => {
                Some(MtlLine::Comment(tokens[1..].join(" ")))
            }
            _ => None,
        }
    }
}
