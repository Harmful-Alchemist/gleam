pub fn main() {
  todo
}

fn sum(xs: List(Int)) -> Int {
  case xs {
    [] -> 0
    [x, ..ys] -> x + sum(ys)
  }
}

pub type Color {
  Red
  Black
  Purple
  Pink
}

pub fn t(x: Color, y: Color, z: Color) {
  case x, y, z {
    _, Red, Black -> 1
    Red, Black, _ -> 2
    _, _, Red -> 3
    _, _, Black -> 4
    _, _, Purple -> 5
    _, _, Pink -> 6
  }
}
