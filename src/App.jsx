import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { preinitModule } from "react-dom";
import { commands } from "./bindings-js-files/index.js";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [host, hostName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("Host", { name }));
  }

  function handleClick() {
    alert("You are hosting " + hostName);
  }
  commands.setStatus("hello", 3000, "Study");

  // This is the ui portion...
  return (
    <main className="container">
      <h1>I am editing this stuff now</h1>

      <div className="row">Images were here. Not commenting..</div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <button
        onClick={async () => {
          commands.print("button1");
          const x = await commands.host("Hello");
          if (x.status === "error") {
            commands.print("error: " + x.error);
          } else {
            commands.print("WHATTHEFUCK");
          }
        }}
      >
        Button 1
      </button>
      <button
        onClick={() => {
          commands.print("button2");
          commands.join("Hello");
        }}
      >
        Button 2
      </button>

      <button
        onClick={async () => {
          commands.print("button3");
          let x = await commands.receivemessage();
          if (x === null) {
            commands.print("NOTHING");
            commands.print(x);
          } else {
            commands.print(x);
          }
        }}
      >
        Button 3
      </button>
      <form
        className="row"
        onSubmit={async (_) => {
          await commands.sendmessage(host);
          commands.print("Sent: " + host);
        }}
      >
        <input
          id="host-button"
          onChange={(e) => hostName(e.currentTarget.value)} // e.currentTarget.value = value of hostName
          placeholder="why does this print..."
        />
        <button onClick={() => {}}>SendMessage</button>
      </form>
      <p>{greetMsg}</p>
    </main>
  );
}

export default App;

/* do backflips in order to get npm working */

/* use "npm run dev" to open up the react thing */

/* Bryce if you are reading this, I probably will not 
edit anything here because it still only opens up as 
a website */

/*     Keeping these as a reference if we need pictures and shit... 
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
*/
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
