export const createRtmpUrl = (key, name) => {
  const host = Cypress.config().baseUrl.includes('localhost')
    ? '0.0.0.0'
    : Cypress.env('host');
  return `rtmp://${host}/${key}/${name}`;
};

export const stopAllTestStreams = () => {
  cy.exec('killall ffmpeg');
};
