import { toList, prepend as listPrepend, CustomType as $CustomType } from "./build/dev/javascript/test_rb_tree/gleam.mjs";


import { toList, prepend as listPrepend, CustomType as $CustomType } from "./gleam.mjs";

export class Zero extends $CustomType { }

export class S extends $CustomType {
  constructor(x0) {
    super();
    this[0] = x0;
  }
}

class Tr extends $CustomType { }

class Fa extends $CustomType { }

function smaller(loop$x, loop$y) {
  while (true) {
    let x = loop$x;
    let y = loop$y;
    if (y instanceof S) {
      if (x instanceof Zero) {
        return new Tr();
      } else {
        let nx = x[0];
        let ny = y[0];
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

function mutual_inner(comp, x, y, ys, lacc, real_acc) {
  if (comp instanceof Tr) {
    return sort_inner(x, ys, listPrepend(y, lacc), real_acc);
  } else {
    return sort_inner(y, ys, listPrepend(x, lacc), real_acc);
  }
}

function sort_inner(loop$element_to_cmp, loop$elements, loop$acc, loop$real_acc) {
  while (true) {
    let element_to_cmp = loop$element_to_cmp;
    let elements = loop$elements;
    let acc = loop$acc;
    let real_acc = loop$real_acc;
    if (elements.atLeastLength(1)) {
      let y = elements.head;
      let x = element_to_cmp;
      let lacc = acc;
      let ys = elements.tail;
      return (() => {
        let comp = smaller(x, y);
        return mutual_inner(comp, x, y, ys, lacc, real_acc);
      })();
    } else if (elements.hasLength(0)) {
      if (acc.atLeastLength(1)) {
        let accs = acc.tail;
        let x = element_to_cmp;
        let ac = acc.head;
        loop$element_to_cmp = ac;
        loop$elements = accs;
        loop$acc = toList([]);
        loop$real_acc = listPrepend(x, real_acc);
      } else {
        let accs = acc.tail;
        let x = element_to_cmp;
        let ac = acc.head;
        loop$element_to_cmp = ac;
        loop$elements = accs;
        loop$acc = toList([]);
        loop$real_acc = listPrepend(x, real_acc);
        //TODO this should be different from above!!
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
    return sort_inner(x, xs, toList([]), toList([]));
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


console.log(main());