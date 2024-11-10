import subprocess

rbt = ["bal_reg.mjs", "bal_fdb.mjs", "bal_qba.mjs", "bal_fdb_switch.mjs", "bal_qba_switch.mjs"]


# commands = ["./mach run --module ../", "deno run ../","bun run ../"] # TODO loop em with warm up?
# 
# print("balance")
# for t in rbt:
#     a = subprocess.run(f"./mach run --module ../{t}", shell=True, check=True, capture_output=True)
#     res = a.stdout.decode().split('\n')[1].strip().split(',')
#     print(f" {float(res[2]) * 1000}, {float(res[0]) * 1000}, {float(res[1]) * 1000}")
#     print("---")

# print("bubble")

# bubble = ["buble_reg.mjs", "buble_reg_decfdb.mjs", "bub_qba.mjs", "buble_reg_decfdbswitch.mjs", "bub_qba_switch.mjs"]
# for t in bubble:
#     a = subprocess.run(f"./mach run --module ../{t}", shell=True, check=True, capture_output=True)
#     res = a.stdout.decode().split('\n')[1].strip().split(',')
#     print(f" {res[2]}, {res[0]}, {res[1]}")
#     print("---")

commands = ["$V8_PATH/out/x64.release/d8 ../", "./mach run --module ../", "bun run ../"]

# commands = ["bun run ../"]

# commands = ["deno run ../"]



for cmd in commands:
    print("***")
    print(cmd)
    print("***")
    print("avg,min,max,warm_up1,warm_up2")

    line = 1
    if cmd == "deno run ../" or cmd == "$V8_PATH/out/x64.release/d8 ../" or cmd == "bun run ../":
        line = 0
    
    print("balance")
    for t in rbt:
        print(f"{t}")
        a = subprocess.run(f"{cmd}{t}", shell=True, check=True, capture_output=True)
        res = a.stdout.decode().split('\n')[line].strip().split(',')
        print(f" {float(res[0]) * 1000}, {float(res[1]) * 1000}, {float(res[2]) * 1000}, {float(res[3]) * 1000}")
        print("---")

    print("bubble")

    bubble = ["buble_reg.mjs", "buble_reg_decfdb.mjs", "bub_qba.mjs", "buble_reg_decfdbswitch.mjs", "bub_qba_switch.mjs"]
    for t in bubble:
        print(f"{t}")
        a = subprocess.run(f"{cmd}{t}", shell=True, check=True, capture_output=True)
        res = a.stdout.decode().split('\n')[line].strip().split(',')
        print(f" {res[0]}, {res[1]}, {res[2]}, {res[3]}")
        print("---")