window.onload = async (_event) => {
  fetch("/status.bin")
    .then((response) => {
      return response.arrayBuffer();
    })
    .then((arraybuf) => {
      var view = new DataView(arraybuf);
      document.getElementById("occupancy").innerText = view.getUint8(0);
    });

  fetch("/history.bin")
    .then((response) => {
      return Promise.all([
        response.arrayBuffer(),
        parseInt(response.headers.get("history-end")),
        parseInt(response.headers.get("history-interval")),
      ]);
    })
    .then(([array, end_timestamp, interval]) => {
      var view = new DataView(array);

      var entries = [];

      for (let i = 0; i < array.byteLength; i++) {
        let value = view.getUint8(i);

        entries.push({
          x: (end_timestamp - i * interval) * 1000,
          y: value == 0xff ? null : value,
        });
      }

      return entries;
    })
    .then((entries) => {
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
