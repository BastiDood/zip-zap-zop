<div class="flex flex-col items-center justify-center p-12">
    <div class="prose max-w-full">
        <h2>Help</h2>
        <p>
            Zip Zap Zop is a very simple game. During each turn, a player has three ways to respond: <strong>Zip</strong
            >,
            <strong>Zap</strong>, or <strong>Zop</strong> (in that order specifically!).
        </p>
        <ul>
            <li>Player A says <strong>Zip</strong> and points to Player C.</li>
            <li>Player C says <strong>Zap</strong> and points to Player B.</li>
            <li>Player B says <strong>Zop</strong> and points to Player C.</li>
            <li>Player C says <strong>Zip</strong> and points to Player A.</li>
            <li>And the cycle goes so on...</li>
        </ul>
        <p>A player is eliminated if:</p>
        <ol>
            <li>They took too long to respond, or</li>
            <li>They responded in violation of the strict Zip-Zap-Zop order.</li>
        </ol>
        <p>
            After each turn, the deadline decays exponentially (e.g., from 10 seconds to 5 seconds to 2.5 seconds and so
            on).
        </p>
        <p>The last player standing wins the game!</p>
        <a class="btn btn-primary text-primary-content" href="/">Okay!</a>
    </div>
</div>
