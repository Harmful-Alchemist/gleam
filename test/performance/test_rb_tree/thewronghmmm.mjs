import { toList, prepend as listPrepend, CustomType as $CustomType } from "./build/dev/javascript/test_rb_tree/gleam.mjs";


export class Zero extends $CustomType { }

export class S extends $CustomType {
  constructor(x0) {
    super();
    this[0] = x0;
  }
}

class Tr extends $CustomType { }

class Fa extends $CustomType { }

function map_acc(loop$xs, loop$f, loop$acc) {
  while (true) {
    let xs = loop$xs;
    let f = loop$f;
    let acc = loop$acc;
    if (xs.atLeastLength(1)) {
      let x = xs.head;
      let ys = xs.tail;
      loop$xs = ys;
      loop$f = f;
      loop$acc = listPrepend(f(x), acc);
    } else if (xs.hasLength(0)) {
      return acc;
    }
  }
}

function smaller(loop$x, loop$y) {
  while (true) {
    let x = loop$x;
    let y = loop$y;
    if (y instanceof S) {
      if (x instanceof Zero) {
        return new Tr();
      } else {
        let ny = y[0];
        let nx = x[0];
        loop$x = nx;
        loop$y = ny;
      }
    } else {
      return new Fa();
    }
  }
}

function lenght(xs) {
  if (xs.hasLength(0)) {
    return new Zero();
  } else if (xs.atLeastLength(1)) {
    let x = xs.head;
    let ys = xs.tail;
    return new S(lenght(ys));
  }
}

function sort_inner(
  loop$element_to_cmp,
  loop$elements,
  loop$acc,
  loop$len,
  loop$real_acc
) {
  while (true) {
    let element_to_cmp = loop$element_to_cmp;
    let elements = loop$elements;
    let acc = loop$acc;
    let len = loop$len;
    let real_acc = loop$real_acc;
    if (elements.atLeastLength(1)) {
      let y = elements.head;
      let ys = elements.tail;
      let x = element_to_cmp;
      let lacc = acc;
      // return (() => {
        let comp = smaller(x, y);
        if (comp instanceof Tr) {
          loop$element_to_cmp = x;
          loop$elements = ys;
          loop$acc = listPrepend(y, lacc);
          loop$len = len;
          loop$real_acc = real_acc;
        } else {
          loop$element_to_cmp = y;
          loop$elements = ys;
          loop$acc = listPrepend(x, lacc);
          loop$len = len;
          loop$real_acc = real_acc;
        }
      // })();
    } else if (elements.hasLength(0)) {
      if (acc.atLeastLength(1)) {
        if (len instanceof S) {
          let ac = acc.head;
          let a = len[0];
          let accs = acc.tail;
          let x = element_to_cmp;
          loop$element_to_cmp = ac;
          loop$elements = accs;
          loop$acc = toList([]);
          loop$len = a;
          loop$real_acc = listPrepend(x, real_acc);
        } else {
          let x = element_to_cmp;
          return listPrepend(x, real_acc);
        }
      } else {
        if (len instanceof S) {
          let ac = acc.head;
          let a = len[0];
          let accs = acc.tail;
          let x = element_to_cmp;
          loop$element_to_cmp = ac;
          loop$elements = accs;
          loop$acc = toList([]);
          loop$len = a;
          loop$real_acc = listPrepend(x, real_acc);
        } else {
          let x = element_to_cmp;
          return listPrepend(x, real_acc);
        }
      }
    }
  }
}

function sort(elements) {
  if (elements.hasLength(0)) {
    return toList([]);
  } else if (elements.atLeastLength(1)) {
    let x = elements.head;
    let xs = elements.tail;
    return (() => {
      let len = lenght(xs);
      return sort_inner(x, xs, toList([]), len, toList([]));
    })();
  }
}

export function main() {
  return sort(
    toList([
      new Zero(),
      new S(new Zero()),
      new S(new S(new Zero())),
      new S(new S(new S(new Zero()))),
      new Zero(),
    ]),
  );
}

let x = main();
console.log(x);