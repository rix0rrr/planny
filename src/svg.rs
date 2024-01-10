use itertools::Itertools;

pub type Coord = u32;

#[derive(Debug, Clone)]
pub struct Svg {
    pub view_box: (Coord, Coord, Coord, Coord),
    pub paths: Vec<Path>,
}

#[derive(Debug, Clone)]
pub struct RenderedSvg {
    pub view_box: (Coord, Coord, Coord, Coord),
    pub rendered_paths: Vec<String>,
}

pub type Path = Vec<Segment>;

#[derive(Debug, Clone)]
pub enum Segment {
    Move(Coord, Coord),
    Line(Coord, Coord),
    Return,
}

impl Svg {
    pub fn render(&self) -> RenderedSvg {
        RenderedSvg {
            view_box: self.view_box,
            rendered_paths: self.paths.iter().map(Self::render_path).collect(),
        }
    }

    fn render_path(path: &Path) -> String {
        path.iter()
            .map(|p| match p {
                Segment::Move(x, y) => format!("M {x} {y}"),
                Segment::Line(x, y) => format!("L {x} {y}"),
                Segment::Return => "Z".to_owned(),
            })
            .join(" ")
    }
}
