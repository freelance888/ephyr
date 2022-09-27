describe('CHECK STREAMING STATE', () => {
  before(() => {
    cy.visit('/');
    cy.deleteAllInputs();
    cy.importJsonConf(Cypress.env('host'));
    cy.runTestStream('rtmp://' + Cypress.env('host') + '/en/primary');
  });

  after(() => {
    cy.deleteAllInputs();
  });

  it('1 Assert Start All', () => {
    cy.allOutputStart();
    cy.wait(6000);
    cy.checkStartedAllStated();
  });

  it('2 Assert Stop All', () => {
    cy.allOutputStop();
    cy.wait(6000);
    cy.checkStoppedAllStated();
  });

  it('3 Click Start All', () => {
    cy.allOutputStart();
    cy.wait(6000);
    cy.checkStartedAllStated();
  });

  it('4 Unselect sidechain', () => {
    cy.get('[data-testid=Teamspeak]')
      .parent()
      .find("input[title='Sidechain']")
      .first()
      .click();
    cy.wait(6000);
  });

  it('4 Assert Started', () => {
    cy.checkStartedAllStated();
  });

  it('5 Select sidechain', () => {
    cy.get('[data-testid=Teamspeak]')
      .parent()
      .find("input[title='Sidechain']")
      .first()
      .click();
    cy.wait(6000);
  });

  it('5 Select Delay should not restart', () => {
    cy.get('[data-testid=Teamspeak]')
      .parent()
      .find("input[title='Delay']")
      .first()
      .type('5.5')
      .trigger('change');
  });

  it('5 Assert Started', () => {
    cy.checkStartedAllStated();
  });

  it('6 Assert Stopped', () => {
    cy.allOutputStop();
    cy.wait(6000);
    cy.checkStoppedAllStated();
  });
});
