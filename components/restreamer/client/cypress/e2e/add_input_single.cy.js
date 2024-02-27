describe('ADD SINGLE INPUT', { testIsolation: false }, () => {
  before(() => {
    cy.visit('/');
    cy.deleteAllInputs();
  });

  after(() => {
    cy.deleteAllInputs();
  });

  it('Add-input', () => {
    cy.get("span:contains('Input')").click();
  });

  it('Set optional label', () => {
    cy.get('[placeholder="optional label"]').type('SINGLE');
  });

  it('Set stream-key', () => {
    cy.get('[placeholder="<stream-key>"]').type('en');
  });

  it('Submits', () => {
    cy.clickAddInputBtn();
  });

  it('Assert', () => {
    cy.get("span:contains('/en/primary'):last").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/en/primary',
    );
  });
});
