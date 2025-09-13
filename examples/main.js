import init, { run_mangle_query } from '../pkg/mangle_wasm_wrapper.js';

async function main() {
    // The init function needs to be called to load the wasm module.
    // It expects the path to the .wasm file.
    try {
        await init('../pkg/mangle_wasm_wrapper_bg.wasm');
    } catch (e) {
        console.error("Error initializing Wasm module. This is expected in this simulated environment because the .wasm file is a placeholder.", e);
        console.log("The rest of the script will not run, but this demonstrates the correct usage pattern.");

        // Even though it fails, let's show the query string and how the function *would* be called.
        const mangleInput = `
          // --- Facts: The ground truth ---
          service("order-service").
          uses_library("order-service", "log4j", "2.14").
          vulnerable_version("log4j", "2.14").

          // --- Rule: How to reason ---
          is_vulnerable(Svc) :-
            uses_library(Svc, Lib, Ver),
            vulnerable_version(Lib, Ver).

          // --- Query: The final question to execute ---
          is_vulnerable(Svc).
        `;

        console.log("\nSample Mangle Input:");
        console.log(mangleInput);
        console.log("\nThis is how you would call the function if initialization succeeded:");
        console.log('const resultJson = run_mangle_query(mangleInput);');

        return; // Stop execution
    }

    // This part of the code will only run if the Wasm module initializes successfully.
    console.log('Wasm module initialized successfully.');

    const mangleInput = `
      // --- Facts: The ground truth ---
      service("order-service").
      uses_library("order-service", "log4j", "2.14").
      vulnerable_version("log4j", "2.14").

      // --- Rule: How to reason ---
      is_vulnerable(Svc) :-
        uses_library(Svc, Lib, Ver),
        vulnerable_version(Lib, Ver).

      // --- Query: The final question to execute ---
      is_vulnerable(Svc).
    `;

    console.log("Running Mangle query...");
    const resultJson = run_mangle_query(mangleInput);
    const result = JSON.parse(resultJson);

    if (result.status === 'success') {
        console.log('Query Succeeded:');
        console.log(result.data);
    } else {
        console.error('Query Failed:');
        console.error(result.message);
    }
}

main();
