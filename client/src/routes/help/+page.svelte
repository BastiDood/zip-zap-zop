<div class="flex h-screen flex-col items-center justify-center gap-4 px-12">
    <div class="space-y-4 text-left">
        <h2 class="text-3xl">Help</h2>
        <p>
            Zip Zap Zop is a very simple game. During each turn, a player has three ways to respond: <strong>Zip</strong
            >,
            <strong>Zap</strong>, or <strong>Zop</strong> (in that order specifically!).
        </p>
        <ul class="list-inside list-disc">
            <li>Player A says <strong>Zip</strong> and points to Player C.</li>
            <li>Player C says <strong>Zap</strong> and points to Player B.</li>
            <li>Player B says <strong>Zop</strong> and points to Player C.</li>
            <li>Player C says <strong>Zip</strong> and points to Player A.</li>
            <li>And the cycle goes so on...</li>
        </ul>
        <p>A player is eliminated if:</p>
        <ol class="list-inside list-decimal">
            <li>They took too long to respond, or</li>
            <li>They responded in violation of the strict Zip-Zap-Zop order.</li>
        </ol>
        <p>
            After each turn, the deadline decays exponentially (e.g., from 10 seconds to 5 seconds to 2.5 seconds and so
            on).
        </p>
        <p>The last player standing wins the game!</p>
        <a class="btn btn-primary" href="/">Okay!</a>
    </div>
</div>
