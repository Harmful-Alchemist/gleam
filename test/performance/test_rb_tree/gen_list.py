import random

def gen_number(x):
    num = "Zero"
    
    for _ in range(x):
        num = f"S({num})"
                
    return num

if __name__ == "__main__":
    lol = "sort(["
    for _ in range(1_000):
        lol += f"{gen_number(random.randrange(50))},"
    lol += "])"
    print(lol)

    # Stack overflow with 10_000 of 1_000 or lower



# Very surprising trees much slower.
# harm@harmbook:~/git/gleam/test/performance$ deno bench -A ./bench_rbt.js --
# Using decision trees: false
# Using qba: false
# Using switch to deduplicate trees: false
# cpu: 11th Gen Intel(R) Core(TM) i7-1165G7 @ 2.80GHz
# runtime: deno 1.45.2 (x86_64-unknown-linux-gnu)

# file:///home/harm/git/gleam/test/performance/bench_rbt.js
# benchmark          time (avg)        iter/s             (min … max)       p75       p99      p995
# ------------------------------------------------------------------- -----------------------------
# Matching RBTs      247.5 ms/iter           4.0 (241.55 ms … 260.06 ms) 253.72 ms 260.06 ms 260.06 ms

# harm@harmbook:~/git/gleam/test/performance$ deno bench -A ./bench_rbt.js -- true
# Using decision trees: true
# Using qba: false
# Using switch to deduplicate trees: false
# cpu: 11th Gen Intel(R) Core(TM) i7-1165G7 @ 2.80GHz
# runtime: deno 1.45.2 (x86_64-unknown-linux-gnu)

# file:///home/harm/git/gleam/test/performance/bench_rbt.js
# benchmark          time (avg)        iter/s             (min … max)       p75       p99      p995
# ------------------------------------------------------------------- -----------------------------
# Matching RBTs     281.23 ms/iter           3.6 (255.19 ms … 300.54 ms) 287.94 ms 300.54 ms 300.54 ms

# harm@harmbook:~/git/gleam/test/performance$ deno bench -A ./bench_rbt.js -- true false true
# Using decision trees: true
# Using qba: false
# Using switch to deduplicate trees: true
# cpu: 11th Gen Intel(R) Core(TM) i7-1165G7 @ 2.80GHz
# runtime: deno 1.45.2 (x86_64-unknown-linux-gnu)

# file:///home/harm/git/gleam/test/performance/bench_rbt.js
# benchmark          time (avg)        iter/s             (min … max)       p75       p99      p995
# ------------------------------------------------------------------- -----------------------------
# Matching RBTs     290.35 ms/iter           3.4 (255.41 ms … 311.47 ms) 303.57 ms 311.47 ms 311.47 ms

# harm@harmbook:~/git/gleam/test/performance$ deno bench -A ./bench_rbt.js --
# Using decision trees: false
# Using qba: false
# Using switch to deduplicate trees: false
# cpu: 11th Gen Intel(R) Core(TM) i7-1165G7 @ 2.80GHz
# runtime: deno 1.45.2 (x86_64-unknown-linux-gnu)

# file:///home/harm/git/gleam/test/performance/bench_rbt.js
# benchmark          time (avg)        iter/s             (min … max)       p75       p99      p995
# ------------------------------------------------------------------- -----------------------------
# Matching RBTs     247.86 ms/iter           4.0     (237 ms … 259.8 ms) 255.48 ms 259.8 ms 259.8 ms

# harm@harmbook:~/git/gleam/test/performance$ 