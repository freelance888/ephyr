import {
  createRtmpUrl,
  runTestStream,
  stopAllTestStreams,
} from '../support/utils';

describe('How to start/stop rtmp streams from test', () => {
  it('starts rtmp stream and stop after 5 sec', () => {
    cy.visit('/');

    // 1. Create new input
    cy.get("span:contains('Input')").click();
    cy.get('[placeholder="optional label"]').type('EN1');
    cy.get('[placeholder="<stream-key>"]').type('en1');
    cy.get("button:contains('Add')").click();
    cy.get("span:contains('/en1/origin'):last").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/en1/origin'
    );

    // 2. Send rtmp stream to specific address
    runTestStream(createRtmpUrl('en1', 'origin'));

    cy.wait(5000);

    // 3. Stops all streams
    stopAllTestStreams();
  });
});
