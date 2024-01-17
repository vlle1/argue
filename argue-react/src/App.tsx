import React from 'react';
import './App.css';
import Game from './game/Game';
import { useState } from "react";

function App() {
  const [statement, setStatement] = useState<string|null>(null);
  const [privatePad, setPrivatePad] = useState<boolean>(false);
  return (
    <div className="App">
      {
        statement ? <Game root_statement={statement} is_private={privatePad}></Game> : <LandingPage setStatement={setStatement} setPrivatePad={setPrivatePad}></LandingPage>
      }
    </div>
  );
}
const LandingPage = ({setStatement, setPrivatePad}: {setStatement: React.Dispatch<React.SetStateAction<string|null>>, setPrivatePad: React.Dispatch<React.SetStateAction<boolean>>}) => {
  return (
    <div className="LandingPage">
      <h1>Welcome to argue-GPT!</h1>
      { process.env.TESTVAR }
      <p>Enter a statement you want to argue for...</p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          const formData = new FormData(e.target as HTMLFormElement);
          setStatement(formData.get("expression")?.toString() || "");
          setPrivatePad(formData.get("private")?.toString() === "private");
        }}
      >
        <input
          type="text"
          placeholder=""
          name="expression"
          className="abs-in"
        ></input>
        <input type="submit" className="landing_submit" value="Join/Create pad"></input>
        <label htmlFor="private"className="landing_private_checkbox"><input type="checkbox" id="private" name="private" value="private"></input>
        make private</label>
        
      </form>
      <h2>
        How to use argue-GPT...
      </h2>
      <p>First, you create a root statement. This is the statement you want to argue for (using ChatGPT as "judge").</p>
      <p>Add more statements using the input field.</p>
      <p><strong>Click on two nodes</strong> to add a "if-then-connection".</p>
      <p><strong>Double-click node</strong> to see if the AI accepts the statement by itself (as "Fact")</p>
      <p><strong>Right-click node</strong> to see if the AI accepts the statement as a logical consequence of all incoming edges.</p>
      <p><strong>Right-click edge</strong> to remove the edge.</p>
      <p>See, if you can convince the AI of your root statement!</p>
      <h2>NOTE</h2>
      <p>This is a PROTOTYPE, further UI may be added. <strong>Open the console</strong>, as some important information (f.ex. AI responses) is only printed there!</p>
    </div>
  );
}

export default App;
