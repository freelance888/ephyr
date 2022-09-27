describe('ADD MULTIPLE MIXIN OUTPUT', () => {
  before(() => {
    cy.visit('/');
    cy.deleteAllInputs();
  });

  after(() => {
    cy.deleteAllInputs();
  });
  it('Add-input', () => {
    cy.get("span:contains('Input')").click();
    cy.get('[placeholder="optional label"]').type('SINGLE');
    cy.get('[placeholder="<stream-key>"]').type('en');
    cy.get('button').contains(/^Add$/).click();
  });

  it('Click Output', () => {
    cy.get("span:contains('Output'):last").click();
    cy.wait(5000);
  });

  it('Set optional label', () => {
    cy.get('[placeholder="optional label"]').type('Teamspeak Multiple Test');
  });

  it('Set rtmp://', () => {
    const urlToPaste = 'rtmp://' + Cypress.env('host') + '/it/some';
    cy.get('[placeholder="rtmp://..."]')
      .invoke('val', urlToPaste)
      .trigger('input');
  });

  it('Set preview', () => {
    const previewToPaste = 'https://www.youtube.com/watch?v=99567P5FiD0';
    cy.get('[placeholder="optional preview url"]')
      .invoke('val', previewToPaste)
      .trigger('input');
  });

  it('Checks a "with mixin" checkbox', () => {
    cy.get('.mix-with > [type="checkbox"]').check();
  });

  it('Set first teamspeak', () => {
    const teamspeakToPaste = 'ts://ts.multiple.com/Multiple1';
    cy.get(
      '[placeholder="ts://<teamspeak-host>:<port>/<channel>?name=<name>&identity=<identity>"]'
    )
      .invoke('val', teamspeakToPaste)
      .trigger('input');
  });

  it('Checks a "with mixin" checkbox', () => {
    cy.get('.mix-with > [type="checkbox"]').check();
  });

  it('Set second teamspeak', () => {
    const teamspeakToPaste = 'ts://ts.multiple.com/Multiple2';
    cy.get(
      '[placeholder="ts://<teamspeak-host>:<port>/<channel>?name=<name>&identity=<identity>"]'
    )
      .eq(1)
      .invoke('val', teamspeakToPaste)
      .trigger('input');
  });

  it('Checks a "with mixin" checkbox', () => {
    cy.get('.mix-with > [type="checkbox"]').check();
  });

  it('Set third teamspeak', () => {
    const teamspeakToPaste = 'ts://ts.multiple.com/Multiple3';
    cy.get(
      '[placeholder="ts://<teamspeak-host>:<port>/<channel>?name=<name>&identity=<identity>"]'
    )
      .eq(2)
      .invoke('val', teamspeakToPaste)
      .trigger('input');
  });

  it('Submits', () => {
    cy.get('button').contains(/^Add$/).click();
    cy.get('button').contains(/^Add$/).should('not.exist');
  });

  it('Only one sidechain checkbox is possible to check through mixins', () => {
    cy.get("span:contains('Teamspeak Multiple Test')")
      .parent()
      .find("input[title='Sidechain']")
      .first()
      .click();
  });

  it('Assert', () => {
    cy.get("span:contains('Teamspeak Multiple Test')").should(
      'have.text',
      'Teamspeak Multiple Test'
    );
    cy.get("span:contains('/Multiple1')").should(
      'have.text',
      'ts://ts.multiple.com/Multiple1'
    );
    cy.get("span:contains('/Multiple2')").should(
      'have.text',
      'ts://ts.multiple.com/Multiple2'
    );
    cy.get("span:contains('/Multiple3')").should(
      'have.text',
      'ts://ts.multiple.com/Multiple3'
    );
    cy.get("span:contains('Teamspeak Multiple Test')")
      .parent()
      .find('input[disabled]')
      .should('have.length', 2);
  });
});
