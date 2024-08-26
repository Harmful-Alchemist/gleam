import random
vs = [1,2,3]
cs = ["Red", "Black"]

def gen_tree(x):
    if x > 2:
        return {"Empty"}
    
    trees = {"Empty"}
    for v in vs:
        for c in cs:
            for t1 in gen_tree(x+1):
                for t2 in gen_tree(x+1):
                    trees.add(f'Node({c}, {v}, {t1}, {t2})')
                
    return trees

if __name__ == "__main__":
    lol = 0
    print("[")
    for v in vs:
        for c in cs:
            for t1 in gen_tree(1):
                for t2 in gen_tree(1):
                    p = random.randrange(10000)
                    if p == 1:
                        print(f"  #({c}, {v}, {t1}, {t2}),")
                    lol = lol + 1
    print("]")
    print(lol)