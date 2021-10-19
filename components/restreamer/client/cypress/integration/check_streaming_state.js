describe('CHECK STREAMING STATE', () => {
  before(() => {
    cy.visit('/');
    cy.deleteAllInputs();
    cy.importJsonConf(Cypress.env('host'));
    cy.runTestStream('rtmp://' + Cypress.env('host') + '/en/origin');
  });

  it('Goes to the homepage', () => {
    cy.visit('/');
  });

  it('Click Start All', () => {
    cy.allOutputStart();
  });

  it('Assert', () => {
    cy.get(
      '[data-icon="circle"][title="Serves failover live RTMP stream"]'
    ).should('have.css', 'color', 'rgb(50, 210, 150)');

    cy.get(
      '[data-icon="arrow-right"][title="Accepts main live RTMP stream"]'
    ).should('have.css', 'color', 'rgb(50, 210, 150)');

    cy.get(
      '[data-icon="arrow-right"][title="Accepts backup live RTMP stream"]'
    ).should('have.css', 'color', 'rgb(50, 210, 150)');

    cy.get(
      '[data-icon="arrow-down"][title="Pulls origin live RTMP stream"]'
    ).should('have.css', 'color', 'rgb(50, 210, 150)');

    cy.get(
      '[data-icon="arrow-right"][title="Accepts origin live RTMP stream"]'
    ).should('have.css', 'color', 'rgb(50, 210, 150)');

    cy.wait(5000);
    cy.get('.status-indicator > [data-icon="circle"]').should(
      'have.css',
      'color',
      'rgb(50, 210, 150)'
    );
  });
});
