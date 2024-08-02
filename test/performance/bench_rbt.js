Deno.bench("Matching RBTs", async (b) => {
    const projectPath = "./test_rb_tree/";
    const treeFeature = Deno.args[0] === "true";
    console.log(treeFeature);

    try {
        await Deno.remove("./test_rb_tree/build/", { recursive: true });
    } catch (err) {
        if (!(err instanceof Deno.errors.NotFound)) {
            throw err;
        }
    }

    const command = treeFeature ?
        new Deno.Command("cargo", {
            args: ["run", "--features", "decisiontree", "--", "build", "--target=js"],
            cwd: projectPath,
        }) : new Deno.Command("cargo", {
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

    b.start();
    const huh = jsCode.main();
    b.end();
    console.log(huh.head["0"]);
});