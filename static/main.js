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
            label: "Gym Occupancy",
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
          elements: {
            point: {
              radius: 0,
            },
          },
          scales: {
            y: {
              ticks: {
                callback: function (value, _index, _ticks) {
                  return value + "%";
                },
              },
            },
            x: {
              type: "time",
              time: {
                displayFormats: {
                  day: "cccc",
                  hour: "HH:00",
                },
                minUnit: "hour",
              },
              ticks: {
                major: { enabled: true },
                source: "auto",
              },
            },
          },
        },
      };

      var chart = new Chart(ctx, config);

      chart.update();
    });
};
