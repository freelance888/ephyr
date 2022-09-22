describe('ADD PULL INPUT', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });
  it('Add-input', () => {
    cy.get('.add-input').click();
  });

  it('Set optional label', () => {
    cy.get('[placeholder="optional label"]').type('PULL');
  });

  it('Set stream-key', () => {
    cy.get('[placeholder="<stream-key>"]').type('ko');
  });

  it('Checks a checkbox', () => {
    cy.get('.pull >> input').check();
  });

  it('Set rtmp://', () => {
    const urlToPaste = 'rtmp://' + Cypress.env('host') + '/en/origin';
    cy.get('[placeholder="rtmp://..."]')
      .invoke('val', urlToPaste)
      .trigger('input');
  });

  it('Submits', () => {
    cy.get('button').contains(/^Add$/).click();
    cy.get('button').contains(/^Add$/).should('not.exist');
  });

  it('Assert', () => {
    cy.get("span:contains('/en/origin'):last").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/en/origin'
    );
  });
});
