var options = {
  showPoint: false,
  lineSmooth: Chartist.Interpolation.simple(),
  axisX: {},
};

window.onload = (event) => {
  fetch("/history")
    .then((response) => {
      console.log(response.status);
      return response.blob();
    })
    .then((blob) => {
      console.log(blob);
      return blob.arrayBuffer();
    })
    .then((buf) => {
      var view = new DataView(buf);

      var array = [];

      for (let i = 0; i < buf.byteLength; i++) {
        array.push(view.getUint8(i));
      }

      var data = {
        labels: [],
        series: [array],
      };

      new Chartist.LineChart(".ct-chart", data, options);
    });
};
