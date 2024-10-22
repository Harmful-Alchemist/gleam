
pub fn main() {
}



pub type Color {
  Red
  Black
}

pub fn t(x: Color,y: Color,z: Color) {
  case x, y, z {
    _, Red, Black -> 1
    Red, Black, _ -> 2
    _, _, Red -> 3
    _, _, Black -> 4
  }
}