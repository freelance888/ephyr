describe('CHECK STREAMING STATE', () => {
  before(() => {
    cy.visit('/');
    cy.deleteAllInputs();
    cy.importJsonConf(Cypress.env('host'));
    cy.runTestStream('rtmp://' + Cypress.env('host') + '/en/origin');
  });

  it('Click Start All', () => {
    cy.allOutputStart();
    cy.wait(5000);
  });

  const greenColor = 'rgb(50, 210, 150)';
  const brownColor = 'rgb(122, 81, 40)';

  it('Assert', () => {
    cy.get(
      '[data-icon="circle"][title="Serves failover live RTMP stream"]'
    ).should('have.css', 'color', greenColor);

    cy.get(
      '[data-icon="arrow-right"][title="Accepts main live RTMP stream"]'
    ).should('have.css', 'color', greenColor);

    cy.get(
      '[data-icon="arrow-right"][title="Accepts backup live RTMP stream"]'
    ).should('have.css', 'color', greenColor);

    cy.get(
      '[data-icon="arrow-down"][title="Pulls origin live RTMP stream"]'
    ).should('have.css', 'color', greenColor);

    cy.get('[data-testid=SINGLE] [data-icon="arrow-right"]').should(
      'have.css',
      'color',
      greenColor
    );

    cy.get('[data-testid=RU] [data-icon="arrow-right"]').should(
      'have.css',
      'color',
      greenColor
    );

    cy.wait(5000);
    cy.get('[data-testid=Teamspeak] [data-icon="circle"]').should(
      'have.css',
      'color',
      greenColor
    );

    cy.get('[data-testid=Twitter] [data-icon="circle"]').should(
      'have.css',
      'color',
      greenColor
    );

    cy.get('[data-testid="[Manual Start] FB"] [data-icon="circle"]').should(
      'have.css',
      'color',
      greenColor
    );

    cy.get('[data-testid="[Manual Start] YT"] [data-icon="dot-circle"]').should(
      'have.css',
      'color',
      brownColor
    );

    cy.get('[data-testid="File Record"] [data-icon="circle"]').should(
      'have.css',
      'color',
      greenColor
    );
  });
});
