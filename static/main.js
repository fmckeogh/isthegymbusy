var options = {
  showPoint: false,
  lineSmooth: Chartist.Interpolation.simple(),
  axisY: {
    scaleMinSpace: 50,
    onlyInteger: true,
  },
  axisX: {
    type: Chartist.FixedScaleAxis,
    divisor: 7,
    labelInterpolationFnc: function (date) {
      return moment(date).format("MMM D");
    },
  },
};

window.onload = (event) => {
  fetch("/history.csv")
    .then((response) => {
      console.log(response.status);
      return response.text();
    })
    .then((text) => {
      console.log(text);

      var entries = text.split("\n").map((s) => {
        var entry = {
          x: moment.unix(parseInt(s.split(" ")[0])).toDate(),
          y: s.split(" ")[1],
        };

        return entry;
      });

      console.log(entries);

      var data = {
        series: [{ data: entries }],
      };

      new Chartist.LineChart(".ct-chart", data, options);
    });
};
