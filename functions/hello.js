export function onRequest(context) {
  fetch("https://sport.wp.st-andrews.ac.uk/").then((response) => {
    return new Response(response.responseText);
  });
}
