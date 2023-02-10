export async function onRequest(context) {
  await fetch("https://sport.wp.st-andrews.ac.uk/").then((response) => {
    return new Response(response.responseText);
  });
}
