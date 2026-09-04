import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { RegisterDto } from "./bindings/RegisterDto";
import { ApiErrorResponse } from "./bindings/ApiErrorResponse";

function App() {
  const [password, setPassword] = useState<string>("");
  const [email, setEmail] = useState<string>("");

  async function register(email: string, password: string) {
    return await invoke<RegisterDto>("register", { email, password });
  }

  async function handleSubmit(e: React.SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    try {
      const result = register(email, password);
      console.log("Registered:", result);
    } catch (err: unknown) {
      const apiError = err as ApiErrorResponse;
      console.log("Registration failed:", apiError);
      console.error(apiError.code, apiError.message);
    }
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <form
        className="row"
        onSubmit={handleSubmit}
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
