console.log("Script running");
document.getElementById('step-btn').addEventListener('click', makeMove);
document.getElementById('reset-btn').addEventListener('click', resetMaze);
document.getElementById('simulate-btn').addEventListener('click', simulateGame);

let qTableData = [];
let currentAgentState = null;

function makeMove() {
    console.log("makeMove called");

    fetch('/maze/step', { method: 'POST' })
        .then(response => response.json())
        .then(data => {
            console.log("Data received from /maze/step", data);
            console.log("Received Q-table data:", data[2]);

            updateMazeDisplay(data[0]); // Update maze display with the new state
            currentAgentState = data[0].agent[0]; // Update the current agent state

            // Check if the game is over
            if (data[1] === "Game over") {
                document.getElementById('status-message').textContent = "Game over";
                document.getElementById('step-btn').disabled = true;
            }

            // Check if the Q-table data is present and is an array
            if(Array.isArray(data[2])) {
                qTableData = data[2]; // Update the global qTableData with the new data
                updateQTable(qTableData); // Update the Q-table display
            } else {
                console.error('Received non-array Q-Table Data:', data[2]);
            }
        })
        .catch(error => console.error('Error during makeMove:', error));
}

function startSimulation() {
    console.log("startSimulation  called");

    const ws = new WebSocket('ws://localhost:3030/ws');

    ws.onopen = function() {
        console.log('WebSocket connection established');
    };

    ws.onmessage = function(event) {
        const data = JSON.parse(event.data);
        updateMazeDisplay(data.current_state);
        qTableData = formatQTableData(data.q_table); // Assuming a function to format Q-table data
        updateQTable(qTableData);
    };

    ws.onclose = function() {
        console.log('WebSocket connection closed');
    };
}

// function formatQTableData(rawData) {
//     // Create an object to hold the aggregated Q-values for each state
//     const formattedData = {};

//     // Iterate over each entry in the raw Q-table data
//     for (const [key, value] of Object.entries(rawData)) {
//         const [x, y, action] = key.split(',').map(Number); // Extract state and action

//         // Initialize state in formattedData if it doesn't exist
//         if (!formattedData[`${x},${y}`]) {
//             formattedData[`${x},${y}`] = {
//                 state: [x, y],
//                 q_values: [null, null, null, null] // Placeholder for 4 actions
//             };
//         }

//         // Map the action to an index (assuming the order Up, Down, Left, Right)
//         const actionIndex = {
//             'Up': 0,
//             'Down': 1,
//             'Left': 2,
//             'Right': 3
//         }[action];

//         // Assign the Q-value to the correct action in the formattedData
//         // formattedData[`${x},${y}`].q_values[actionIndex] = value;
//         formattedData[`${x},${y}`].q_values[actionIndex] = value ?? null; // Use null-coalescing for safety
//     }

//     // Convert the formattedData object back into an array
//     return Object.values(formattedData);
// }

function formatQTableData(rawData) {
    const formattedData = {};
    const actionMap = ['Up', 'Down', 'Left', 'Right'];

    for (const [key, value] of Object.entries(rawData)) {
        const [x, y, actionNum] = key.split(',').map(Number);

        if (!formattedData[`${x},${y}`]) {
            formattedData[`${x},${y}`] = {
                state: [x, y],
                q_values: [null, null, null, null]
            };
        }

        // Map numeric action to string
        const action = actionMap[actionNum];
        if(action !== undefined) {
            const actionIndex = actionMap.indexOf(action);
            formattedData[`${x},${y}`].q_values[actionIndex] = value;
        }
    }

    return Object.values(formattedData);
}


function simulateGame() {
    console.log("Simulate game called");

    // Establish a WebSocket connection
    const ws = new WebSocket('ws://localhost:3030/maze/simulate');

    ws.onopen = function() {
        console.log('WebSocket connection established');
        // Optionally, send a message to the server to start the simulation
        ws.send(JSON.stringify({ action: 'startSimulation' }));
    };

    ws.onmessage = function(event) {
        console.log('WebSocket message received:', event.data); // Log incoming message

        const data = JSON.parse(event.data);
        updateMazeDisplay(data.current_state); // Update maze display with the new state
        qTableData = formatQTableData(data.q_table); // Format and update the Q-table display
        updateQTable(qTableData);

        // Check if the game is over
        if (data.game_over) {
            console.log('Simulation complete');
            ws.close(); // Close the WebSocket connection
        }
    };

    ws.onclose = function() {
        console.log('WebSocket connection closed');
    };

    ws.onerror = function(error) {
        console.error('WebSocket error:', error);
    };
}



function resetMaze() {
    console.log("resetMaze called");

    fetch('/maze/reset', { method: 'POST' })
        .then(response => response.json())
        .then(data => {
            console.log("Data received from /maze/reset", data);
            updateMazeDisplay(data);
            document.getElementById('status-message').textContent = "Maze reset";
            document.getElementById('step-btn').disabled = false;
            currentAgentState = data.agent[0];
        })
        .catch(error => console.error('Error during resetMaze:', error));
}


function updateMazeDisplay(mazeData) {
    console.log("updateMazeDisplay called with data:", mazeData);

    const mazeContainer = document.getElementById('maze-container');
    mazeContainer.innerHTML = ''; // Clear existing maze

    for (let row = 0; row < 5; row++) {
        for (let col = 0; col < 5; col++) {
            const cell = document.createElement('div');
            cell.classList.add('cell');

            if (mazeData.agent[0][0] === row && mazeData.agent[0][1] === col) {
                cell.classList.add('agent');
            } else if (mazeData.obstacles.some(obstacle => obstacle[0] === row && obstacle[1] === col)) {
                cell.classList.add('obstacle');
            } else if (mazeData.goal[0] === row && mazeData.goal[1] === col) {
                cell.classList.add('goal');
            }

            mazeContainer.appendChild(cell);
        }
    }
    // Update the Q-value table display
    updateQTable(qTableData);
}

function updateQTable(qTable) {
    console.log("updateQTable called with data:", qTable);

    if (!Array.isArray(qTable)) {
        console.error('updateQTable was passed a non-array value', qTable);
        return; // Exit the function early
    }

    const qTableContainer = document.getElementById('q-table'); // Updated container ID
    qTableContainer.innerHTML = ''; // Clear existing Q-value table

    const table = document.createElement('table');
    table.classList.add('q-table');

    // Create table header
    const headerRow = document.createElement('tr');
    const headerCell = document.createElement('th');
    headerCell.textContent = 'State';
    headerRow.appendChild(headerCell);

    for (const action of ['Up', 'Down', 'Left', 'Right']) {
        const actionHeaderCell = document.createElement('th');
        actionHeaderCell.textContent = action;
        headerRow.appendChild(actionHeaderCell);
    }

    table.appendChild(headerRow);

    // Create table rows for each state
    for (const qTableEntry of qTableData) {
        const stateRow = document.createElement('tr');
        const stateCell = document.createElement('td');
        stateCell.setAttribute('data-state', qTableEntry.state.join());
        stateCell.textContent = `(${qTableEntry.state[0]}, ${qTableEntry.state[1]})`;
        stateRow.appendChild(stateCell);

        for (const qValue of qTableEntry.q_values) { // Updated key to q_values
            const qValueCell = document.createElement('td');
            qValueCell.textContent = qValue !== null ? qValue.toFixed(2) : 'N/A';
            stateRow.appendChild(qValueCell);
        }

        table.appendChild(stateRow);
    }

    qTableContainer.appendChild(table);

    // Highlight the current state
    if (currentAgentState) {
        const currentCell = document.querySelector(`#q-table td[data-state="${currentAgentState.join()}"]`);
        if (currentCell) {
            currentCell.classList.add('current-state');
        }
    }    
}



// Call updateMazeDisplay and updateQTable on page load
window.onload = function() {
    console.log("window.onload called");
    fetch('/state')
        .then(response => response.json())
        .then(data => {
            console.log("Data received from /state", data);
            updateMazeDisplay(data[0]);
            qTableData = data[1];
            updateQTable(qTableData);
        })
        .catch(error => console.error('Error during window.onload:', error));
};

