window.onload = async (_e) => {
  const response = await fetch("/status");
  const array = await response.arrayBuffer();
  update_status(new DataView(array).getUint8(0));

  await display_data("chart-today", "/history/today");
  await display_data("chart-average", "/history/average");
};

function update_status(occupancy) {
  document.getElementById("occupancy").innerText = occupancy;

  var answer = document.getElementById("answer");

  if (occupancy >= 50) {
    answer.innerText = "Yes";
    answer.classList.remove("text-success");
    answer.classList.add("text-danger");
  } else {
    answer.innerText = "No";
    answer.classList.remove("text-danger");
    answer.classList.add("text-success");
  }
}

async function display_data(id, data_path) {
  const response = await fetch(data_path);

  const array = await response.arrayBuffer();
  const view = new DataView(array);

  const latest_timestamp = response.headers.get("history-latest");
  const interval = response.headers.get("history-interval");

  var entries = [];

  for (let i = 0; i < array.byteLength; i++) {
    const value = view.getUint8(i);

    entries.push({
      x: (latest_timestamp - i * interval) * 1000,
      y: value == 0xff ? null : value,
    });
  }

  create_chart(id, entries);
}

function create_chart(id, data) {
  new Chart(id, {
    type: "line",
    data: {
      datasets: [
        {
          parsing: false,
          data,
        },
      ],
    },
    options: {
      maintainAspectRatio: false,
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
              size: 28,
            },
            callback: function (value, _a, _b) {
              return value + "%";
            },
          },
          min: 0,
          max: 120,
        },
        x: {
          type: "time",
          time: {
            minUnit: "hour",
          },
          min: "6:00",
          max: "21:00",
          grid: {
            lineWidth: (context) => (context.tick.major ? 3 : 1),
            color: (context) =>
              context.tick.major ? "#A5A5A5" : Chart.defaults.borderColor,
          },
          ticks: {
            major: { enabled: true },
            source: "auto",
            font: {
              size: 28,
            },
            callback: function (value, _a, _b) {
              let time = luxon.DateTime.fromMillis(value);

              if (time.hour % 3 == 0) {
                return time.hour + ":00";
              }

              return null;
            },
          },
        },
      },
    },
  });
}
