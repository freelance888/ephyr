describe('ADD BACKUP INPUT', () => {
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
    cy.get('[placeholder="optional label"]').type('BACKUP');
  });

  it('Set stream-key', () => {
    cy.get('[placeholder="<stream-key>"]').type('it');
  });

  it('Add backups and submit', () => {
    cy.clickAddBackupBtn();
    cy.clickAddBackupBtn();
    cy.clickAddInputBtn();
  });

  it('Assert', () => {
    cy.get("span:contains('/it/primary')").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/it/primary'
    );
    cy.get("span:contains('/it/backup1')").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/it/backup1'
    );
    cy.get("span:contains('/it/backup2')").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/it/backup2'
    );
  });
});
