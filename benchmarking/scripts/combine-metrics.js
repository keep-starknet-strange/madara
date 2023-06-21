const fs = require("fs");

// the scripts takes metrics_erc20.json and metrics_erc721.json and combines them into one file metrics.json
// this allows both metrics to be shown on the same page

function main() {
  const fileNames = [
    "reports/metrics_erc20.json",
    "reports/metrics_erc721.json",
  ];
  const finalOutput = [];
  fileNames.forEach((fileName) => {
    const jsonString = fs.readFileSync(fileName);
    const metrics = JSON.parse(jsonString);
    metrics.forEach((metric) => finalOutput.push(metric));
  });

  fs.writeFileSync("reports/metrics.json", JSON.stringify(finalOutput));
}

try {
  main();
} catch (err) {
  console.log(err);
  process.exit(-1);
}
