const treeFeature = Deno.args[0] === "true";
console.log(`Using decision trees: ${treeFeature}`);

const qbaFeature = Deno.args[1] === "true";
console.log(`Using qba: ${qbaFeature}`);

const switchFeature = Deno.args[2] === "true";
console.log(`Using switch to deduplicate trees: ${switchFeature}`);

Deno.bench("Matching RBTs", async (b) => {
    const projectPath = "./test_rb_tree/";

    try {
        await Deno.remove("./test_rb_tree/build/", { recursive: true });
    } catch (err) {
        if (!(err instanceof Deno.errors.NotFound)) {
            throw err;
        }
    }

    const command = treeFeature ?
        qbaFeature ?
            switchFeature ? new Deno.Command("cargo", {
                args: ["run", "--features", "decisiontree_switch,qba", "--", "build", "--target=js"],
                cwd: projectPath,
            }) :
                new Deno.Command("cargo", {
                    args: ["run", "--features", "decisiontree,qba", "--", "build", "--target=js"],
                    cwd: projectPath,
                })
            :
            switchFeature ? new Deno.Command("cargo", {
                args: ["run", "--features", "decisiontree_switch", "--", "build", "--target=js"],
                cwd: projectPath,
            }) :
                new Deno.Command("cargo", {
                    args: ["run", "--features", "decisiontree", "--", "build", "--target=js"],
                    cwd: projectPath,
                })
        :
        new Deno.Command("cargo", {
            args: ["run", "--", "build", "--target=js"],
            cwd: projectPath,
        });

    const output = await command.output();
    const { code } = output;

    if (code !== 0) {
        console.log(new TextDecoder().decode(output.stdout));
        console.log(new TextDecoder().decode(output.stderr));
        throw "failed to compile";
    }

    const jsPath = "./test_rb_tree/build/dev/javascript/test_rb_tree/test_rb_tree.mjs";
    const jsCode = await import(jsPath);
    // const lol = [];
    let huh;
    b.start();
    // for (let x = 0; x < 100; x++) {
    // lol.push(jsCode.main());
    // }
    huh = jsCode.main();
    b.end();
    // const huh = lol[0];
    // console.log(lol);
    // console.log(huh);
    // let list = huh;
    // while (list.head) {
    //     console.log(list.head);
    //     list = list.tail;
    // }
    // console.log("====")
});