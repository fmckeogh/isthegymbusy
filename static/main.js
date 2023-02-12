window.onload = (event) => {
  fetch("/history.csv")
    .then((response) => {
      return response.text();
    })
    .then((text) => {
      var entries = text.split("\n").map((s) => {
        return {
          x: parseInt(s.split(" ")[0]) * 1000,
          y: parseInt(s.split(" ")[1]),
        };
      });

      var data = {
        datasets: [
          {
            parsing: false,
            data: entries,
          },
        ],
      };

      const ctx = document.getElementById("chart");

      const config = {
        type: "line",
        data: data,
        options: {
          responsive: true,
          plugins: {
            legend: {
              display: false,
            },
          },
          elements: {
            point: {
              radius: 0,
            },
            line: {
              borderWidth: 5,
            },
          },
          scales: {
            y: {
              ticks: {
                font: {
                  size: 24,
                },
                callback: function (value, _index, _ticks) {
                  return value + "%";
                },
              },
            },
            x: {
              type: "time",
              time: {
                minUnit: "hour",
              },
              grid: {
                lineWidth: (context) => (context.tick.major ? 3 : 1),
                color: (context) =>
                  context.tick.major ? "#A5A5A5" : Chart.defaults.borderColor,
              },
              ticks: {
                major: { enabled: true },
                source: "auto",
                font: {
                  size: 24,
                },
                callback: function (value, index, ticks) {
                  let time = luxon.DateTime.fromMillis(value);

                  if (time.hour == 0) {
                    return time.toFormat("cccc");
                  }

                  if (time.hour % 3 == 0) {
                    return time.hour + ":00";
                  }

                  return null;
                },
              },
            },
          },
        },
      };

      var chart = new Chart(ctx, config);

      chart.update();
    });
};
