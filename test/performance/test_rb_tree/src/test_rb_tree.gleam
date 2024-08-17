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

  let cases = [
    #(R, R, R),
    #(R, R, B),
    #(R, R, G),
    #(R, B, R),
    #(R, B, B),
    #(R, B, G),
    #(R, G, R),
    #(R, G, B),
    #(R, G, G),
    #(B, R, R),
    #(B, R, B),
    #(B, R, G),
    #(B, B, R),
    #(B, B, B),
    #(B, B, G),
    #(B, G, R),
    #(B, G, B),
    #(B, G, G),
    #(G, R, R),
    #(G, R, B),
    #(G, R, G),
    #(G, B, R),
    #(G, B, B),
    #(G, B, G),
    #(G, G, R),
    #(G, G, B),
    #(G, G, G),
  ]

  map(cases, idontknow_tuple)
  let f = fn(x) { x }
  map_acc([], f, [])

  let cases = [
    #(Red, Red, Red),
    #(Red, Red, Black),
    #(Red, Black, Red),
    #(Red, Black, Black),
    #(Black, Red, Red),
    #(Black, Red, Black),
    #(Black, Black, Red),
    #(Black, Black, Black),
  ]

  map(cases, maranget2_tuple)
}

// fn maranget1(x, y) {
//   case x, y {
//     Red, _ -> 1
//     _, Red -> 2
//     Black, Black -> 3
//   }
// }

fn maranget2(x, y, z) {
  case x, y, z {
    _, Red, Black -> 1
    Red, Black, _ -> 2
    _, _, Red -> 3
    _, _, Black -> 4
  }
}

fn maranget2_tuple(x: #(Color, Color, Color)) {
  maranget2(x.0, x.1, x.2)
}

fn idontknow_tuple(x: #(RBG, RBG, RBG)) {
  idontknow(x.0, x.1, x.2)
}

fn idontknow(x, y, z) {
  case x, y, z {
    R, B, G -> 1
    R, B, _ -> 2
    _, _, G -> 3
    _, R, _ -> 4
    B, _, _ -> 5
    G, _, _ -> 6
    R, _, _ -> 7
  }
  // _, _, _ -> 10
}

fn hmm(x, y, z) {
  case x, y, z {
    Red, Red, _ -> 1
    Black, Red, _ -> 2
    _, Black, _ -> 3
  }
}

fn hmm2(x, y, z) {
  case x, y, z {
    Red, Red, Red -> 1
    Black, Red, _ -> 2
    _, Black, _ -> 3
    _, _, _ -> 4
  }
}

fn hmm3(x) {
  case x {
    R -> "R"
    B -> "B"
    G -> "G"
  }
}

pub type RBG {
  R
  B
  G
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
  }
  // _ -> acc TODO!
}
