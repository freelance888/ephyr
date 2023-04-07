describe('ADD PULL INPUT', { testIsolation: false },() => {
  before(() => {
    cy.visit('/');
    cy.deleteAllInputs();
  });

  after(() => {
    cy.deleteAllInputs();
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
    const urlToPaste = 'rtmp://' + Cypress.env('host') + '/en/primary';
    cy.get('[placeholder="rtmp://..."]')
      .invoke('val', urlToPaste)
      .trigger('input');
  });

  it('Submits', () => {
    cy.clickAddInputBtn();
  });

  it('Assert', () => {
    cy.get("span:contains('/en/primary'):last").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/en/primary'
    );
  });
});
