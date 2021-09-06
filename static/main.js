let messagesDiv = document.getElementById("messages");

// websocket things
let secureAdd = "";
if(document.location.protocol === "https:") {
    secureAdd = "s";
}
let webSocket = new WebSocket("ws" + secureAdd + "://" + window.location.host + "/movews");
let gameIdInput = document.getElementById("gameIdInput");
let form = document.getElementById("moveForm");
form.addEventListener("submit", submitAction);
function submitAction(event) {
    event.preventDefault();
    webSocket.send(gameIdInput.value.toString());
}

let board;
let boardInternal = new Chess();

// if expecting color eg white, black
let expectingColor = true;
let color = "";
webSocket.onmessage = function(event) {
    if(expectingColor) {
        color = event.data;
        expectingColor = false;
        form.parentNode.removeChild(form);
        let boardNode = document.getElementById("board");
        boardNode.style.height = (window.innerHeight - 50) + "px";
        boardNode.style.width = (window.innerHeight - 50) + "px";
        board = initChessground(document.getElementById("board"), color);
    } else {
        const orig = event.data.substring(0, 2);
        const dest = event.data.substring(2);
        board.move(orig, dest);
        boardInternal.move({ from: orig, to: dest });
        updateChessground(board);
        board.playPremove();
        updateChessground(board);
    }
}

//chessground
function initChessground(elem, color) {
    let board = Chessground(elem, {}); // empty options now, but overwritten in updateChessground
    updateChessground(board);
    return board;
}

// called on a move, any color, does stuff like update valid moves on chessground
function updateChessground(board) {
    const columns = ["a", "b", "c", "d", "e", "f", "g", "h"];
    const files   = [  1,   2,   3,   4,   5,   6,   7,   8];
    let allMoves = new Map();
    for(const col of columns) {
        for(const file of files) {
            const square = col + file;
            const squareMoves = boardInternal.moves({ square: square, verbose: true }).map(x => x.to);
            if(squareMoves.length > 0) {
                allMoves.set(square, squareMoves);
            }
        }
    }
    board.set(
        {
            turnColor: boardInternal.turn() === "w" ? "white" : "black",
            orientation: color,
            movable: {
                free: false,
                color: color,
                dests: allMoves,
                events: {
                    after: function(orig, dest) { webSocket.send(orig + dest); boardInternal.move({ from: orig, to: dest }); }
                }
            },
        }
    );
}
