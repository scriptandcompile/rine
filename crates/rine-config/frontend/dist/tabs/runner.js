// Runner tab: execute and monitor running executables with stdout/stderr output

function setupRunnerTab() {
  document.getElementById("launch-btn").addEventListener("click", async () => {
    if (!exePath) return;
    const stdoutBox = document.getElementById("runner-stdout");
    const stderrBox = document.getElementById("runner-stderr");
    const exitDisplay = document.getElementById("exit-code-display");
    stdoutBox.innerHTML = "Launching…";
    stderrBox.innerHTML = "";
    exitDisplay.classList.add("hidden");
    try {
      const { stdout, stderr, exit_code } = await window.__TAURI__.core.invoke("launch_exe", { exePath });
      stdoutBox.innerHTML = stdout ? ansiToHtml(stdout) : "<em>no output</em>";
      stderrBox.innerHTML = stderr ? ansiToHtml(stderr) : "<em>no output</em>";
      exitDisplay.textContent = "exit " + exit_code;
      exitDisplay.className = "exit-code" + (exit_code !== 0 ? " error" : "");
    } catch (err) {
      stdoutBox.textContent = "Error: " + err;
      stderrBox.innerHTML = "";
    }
  });

  // Runner sub-tabs (stdout/stderr switching)
  for (const btn of document.querySelectorAll(".runner-tab")) {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".runner-tab").forEach(b => b.classList.remove("active"));
      document.querySelectorAll(".runner-pane").forEach(p => p.classList.remove("active"));
      btn.classList.add("active");
      document.getElementById("runner-" + btn.dataset.output).classList.add("active");
    });
  }
}
