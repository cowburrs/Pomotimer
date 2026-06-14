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

<<<<<<< HEAD
// This is the ui portion... 
  return (
    <main className="container">
      <h1>I am editing this stuff now</h1>
  
      <div className="row">
        Images were here. Not commenting.. 
      </div>
=======
  console.log("testing to see if javascript works in this function");
  return (
    <main className="container">
      <h1>I am editing this stuff now</h1>

      <div className="row">Images were here. Not commenting..</div>
>>>>>>> 52a817edf04b3d8ed63098989407459cbf3df1d4
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {onabort
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="host-button"
          onChange={(e) => hostName(e.currentTarget.value)}
          placeholder="does this print????"
        />
        <button type="submit">Host</button>
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
<<<<<<< HEAD
 // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/








=======
>>>>>>> 52a817edf04b3d8ed63098989407459cbf3df1d4
