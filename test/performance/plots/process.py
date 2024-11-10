class NewMeas:
    def __init__(self, name, avg, minn, maxx, warmup):
        self.name = name
        self.avg = avg
        self.minn = minn
        self.maxx = maxx
        self.warmup = warmup

    def __str__(self):
        return f"NM {self.avg}, {self.minn}, {self.maxx}, {self.warmup}"
    
    def __repr__(self):
        return self.__str__()

dict = {}

with open('./outcomes_single_warm_up.txt', 'r') as file:
    for x in ["V8", "SpiderMonkey", "Bun"]:
        dict[x] = {}
        for a in ["balance", "bubble"]:
            dict[x][a] = {}
            for y in ["regular", "tree with fdb", "tree with qba", "deduplicated tree with fdb","deduplicated tree with qba"]:
                line: str = file.readline()
                count = 0
                # print("uh")
                # print(line)
                while (not ',' in line or 'avg' in line): 
                    line = file.readline()
                m = list(map(lambda z: float(z),line.strip().split(',')))

                dict[x][a][y] = NewMeas(y, m[0], m[1], m[2], m[3])


# rows = ""

# for x in dict:
#     rows += f'\\multirow{{5}}{{6em}}{{{x}}}'
#     for y in dict[x]["bubble"]:
#         m = dict[x]["bubble"][y]
#         rows += f'  & {y} & {m.warmup:.2f} & {m.avg:.2f} & {m.minn:.2f} & {m.maxx:.2f} \\\\ \n'
#     if x != "Bun":
#         rows += '\\hline \n'

# print(
# f'''
# \\begin{{table*}}\centering
# \\begin{{tabular}}{{@{{}}ll|cccc@{{}}}} \\toprule
# JS engine & Program & Warm-up & Average & Minimum & Maximum \\\\
# \\midrule
# {rows}
# \\bottomrule
# \\end{{tabular}}
# \\caption{{Bubble sort performance measurements in milliseconds}}
# \\label{{table:results_test_bubble}}
# \\end{{table*}}
# '''
# )


# rows = ""

# for x in dict:
#     rows += f'\\multirow{{5}}{{6em}}{{{x}}}'
#     for y in dict[x]["balance"]:
#         m = dict[x]["balance"][y]
#         rows += f'  & {y} & {m.warmup:.2f} & {m.avg:.2f} & {m.minn:.2f} & {m.maxx:.2f} \\\\ \n'
#     if x != "Bun":
#         rows += '\\hline \n'

# print(
# f'''
# \\begin{{table*}}\centering
# \\begin{{tabular}}{{@{{}}ll|cccc@{{}}}} \\toprule
# JS engine & Program & Warm-up & Average & Minimum & Maximum \\\\
# \\midrule
# {rows}
# \\bottomrule
# \\end{{tabular}}
# \\caption{{Balancing red-black trees performance measurements in microseconds}}
# \\label{{table:results_test_bubble}}
# \\end{{table*}}
# '''
# )