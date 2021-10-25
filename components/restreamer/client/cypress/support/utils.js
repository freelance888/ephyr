export const runTestStream = (rtmp) => {
  cy.exec(
    `ffmpeg -stream_loop -1 -re -nostdin -i ./cypress/data/test_video.mp4 -vcodec libx264 -preset:v ultrafast -acodec aac -f flv ${rtmp} >/dev/null 2>&1 &`
  );
};

export const createRtmpUrl = (key, name) => {
  const host = Cypress.config().baseUrl.includes('localhost')
    ? '0.0.0.0'
    : Cypress.env('host');
  return `rtmp://${host}/${key}/${name}`;
};

export const stopAllTestStreams = () => {
  cy.exec('killall ffmpeg');
};
