describe('ADD SINGLE OUTPUT', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });
  it('Click Output', () => {
    cy.get("span:contains('Output'):last").click();
    cy.wait(5000);
  });

  it('Set optional label', () => {
    cy.get('[placeholder="optional label"]').type('Twitter');
  });

  it('Set rtmp://', () => {
    const urlToPaste = 'rtmp://' + Cypress.env('host') + '/it/main';
    cy.get('[placeholder="rtmp://..."]')
      .invoke('val', urlToPaste)
      .trigger('input');
  });

  it('Set preview', () => {
    const previewToPaste = 'https://creativesociety.com/ru';
    cy.get('[placeholder="optional preview url"]')
      .invoke('val', previewToPaste)
      .trigger('input');
  });

  it('Submits', () => {
    cy.get("button:contains('Add')").click();
    cy.get("button:contains('Add')").should('not.exist');
  });

  it('Assert', () => {
    cy.get("span:contains('Twitter')").should('have.text', 'Twitter');
    cy.get("span:contains('/it/main'):last").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/it/main'
    );
  });
});
