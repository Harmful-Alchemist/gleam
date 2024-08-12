pub fn main() {
  let cases = [
    #(Red, 12, Empty, Empty),
    #(Black, 12, Node(Red, 1, Empty, Empty), Node(Red, 20, Empty, Empty)),
    #(Black, 12, Node(Black, 1, Empty, Empty), Node(Black, 20, Empty, Empty)),
    #(
      Black,
      12,
      Node(Red, 1, Empty, Empty),
      Node(Red, 20, Empty, Node(Red, 21, Empty, Empty)),
    ),
  ]

  map(cases, balance_tuple)
  // let f = fn(x) { x }
  // map_acc([], f, [])
}

pub type Color {
  Red
  Black
}

pub type RBT(t) {
  Node(Color, t, RBT(t), RBT(t))
  Empty
}

fn balance(c, v, t1, t2) {
  case c, v, t1, t2 {
    Black, z, Node(Red, y, Node(Red, x, a, b), c1), d
    | Black, z, Node(Red, x, a, Node(Red, y, b, c1)), d
    | Black, x, a, Node(Red, z, Node(Red, y, b, c1), d)
    | Black, x, a, Node(Red, y, b, Node(Red, z, c1, d)) ->
      Node(Red, y, Node(Black, x, a, b), Node(Black, z, c1, d))
    a, b, c2, d -> Node(a, b, c2, d)
  }
  // case t1 {
  //   Node(Red, y, Node(Red, x, a, b), c) -> 1 //Ah cool already wrong!
  //   r -> 2
  // }
}

fn balance_tuple(x: #(Color, t, RBT(t), RBT(t))) {
  balance(x.0, x.1, x.2, x.3)
}

fn map(xs, f) {
  map_acc(xs, f, [])
}

fn map_acc(xs, f, acc) {
  case xs {
    
    [x, ..ys] -> map_acc(ys, f, [f(x), ..acc])
    [] -> acc
    // _ -> acc TODO!
  }
}
