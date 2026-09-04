import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [password, setPassword] = useState<string>("");
  const [email, setEmail] = useState<string>("");

  // async function greet() {
  //   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //   setGreetMsg(await invoke("greet", { name }));
  // }
  async function register(email: string, password: string) {
    await invoke("register", { email, password });
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          register(email, password);
        }}
      >
        <input
          id="email-input"
          onChange={(e) => setEmail(e.currentTarget.value)}
          placeholder="Email"
        />
        <input
          id="password-input"
          onChange={(e) => setPassword(e.currentTarget.value)}
          placeholder="Password"
        />
        <button type="submit">Register</button>
      </form>
    </main>
  );
}

export default App;
