const express = require('express');
const app = express();
const port = 3000;

app.use(function (req, res, next) {
    res.header("Cross-Origin-Embedder-Policy", "require-corp");
    res.header("Cross-Origin-Opener-Policy", "same-origin");
    next();
});

app.use(express.static('.'));

app.listen(port, () => {
    console.log(`Example app listening on port ${port}`);
})