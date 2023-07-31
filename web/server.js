const express = require('express');
const app = express();
const port = process?.env?.PORT || 3000;

app.use(function (req, res, next) {
    console.log(req.url);
    res.header("Cross-Origin-Embedder-Policy", "require-corp");
    res.header("Cross-Origin-Opener-Policy", "same-origin");
    next();
});

app.use(express.static('.', {
    etag: false,
    maxAge: '1'
}));

app.listen(port, () => {
    console.log(`Example app listening on port ${port}`);
})