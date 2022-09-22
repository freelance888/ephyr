describe('ADD MIXIN OUTPUT', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });
  it('Click Output', () => {
    cy.get("span:contains('Output'):last").click();
    cy.wait(5000);
  });

  it('Set optional label', () => {
    cy.get('[placeholder="optional label"]').type('Teamspeak');
  });

  it('Set rtmp://', () => {
    const urlToPaste = 'rtmp://' + Cypress.env('host') + '/it/backup';
    cy.get('[placeholder="rtmp://..."]')
      .invoke('val', urlToPaste)
      .trigger('input');
  });

  it('Set preview', () => {
    const previewToPaste = 'https://www.youtube.com/watch?v=99567P5FiD0';
    cy.get('[placeholder="optional preview url"]')
      .invoke('val', previewToPaste)
      .trigger('input');
  });

  it('Checks a checkbox', () => {
    cy.get('.mix-with > [type="checkbox"]').check();
  });

  it('Set teamspeak', () => {
    const teamspeakToPaste = 'ts://ts.single.com/Single';
    cy.get(
      '[placeholder="ts://<teamspeak-host>:<port>/<channel>?name=<name>&identity=<identity>"]'
    )
      .invoke('val', teamspeakToPaste)
      .trigger('input');
  });

  it('Submits', () => {
    cy.get('button').contains(/^Add$/).click();
    cy.get('button').contains(/^Add$/).should('not.exist');
  });

  it('Assert', () => {
    cy.get("span:contains('Teamspeak')").should('have.text', 'Teamspeak');
    cy.get("span:contains('/Single')").should(
      'have.text',
      'ts://ts.single.com/Single'
    );
  });
});
