describe('ADD SINGLE INPUT', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
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
    cy.get('button').contains(/^Add$/).click();
  });

  it('Assert', () => {
    cy.get("span:contains('/en/primary'):last").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/en/primary'
    );
  });
});
