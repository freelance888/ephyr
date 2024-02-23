describe('CHECK FILE RECORDING', { testIsolation: false }, () => {
  const COMMON_WAIT_TIME = 10000;

  before(() => {
    cy.visit('/');
    cy.stopAllTestStreams();
    cy.deleteAllInputs();
    cy.importJsonConf(Cypress.env('host'));
    cy.runTestStream('rtmp://' + Cypress.env('host') + '/en/primary');
  });

  after(() => {
    cy.deleteAllInputs();
  });

  it('Start streams 2 times to create 2 file records', () => {
    cy.allOutputStart();
    cy.wait(COMMON_WAIT_TIME);
    cy.allOutputStop();

    cy.allOutputStart();
    cy.wait(COMMON_WAIT_TIME);
    cy.allOutputStop();
  });

  it('Open file modal', () => {
    cy.get('[data-testid="File Record"] .output-mixes a').click();
  });

  it('Should be 2 file records', () => {
    cy.get('.record').should('have.length', 2);
  });

  it('Remove 1 file record', () => {
    cy.get('.record').first().find('[title="Remove recorded file"]').click();
  });

  it('Should be 1 file record', () => {
    cy.get('.record').should('have.length', 1);
  });

  it('File record should be downloaded', () => {
    cy.get('.record').first().find('[title="Download recorded file"]').click();
    cy.get('.uk-modal-dialog .uk-close').eq(0).click();
  });
});
