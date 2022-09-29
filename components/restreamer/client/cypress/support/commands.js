// ***********************************************
// This example commands.js shows you how to
// create various custom commands and overwrite
// existing commands.
//
// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************
//
//
// -- This is a parent command --
// Cypress.Commands.add('login', (email, password) => { ... })
//
//
// -- This is a child command --
// Cypress.Commands.add('drag', { prevSubject: 'element'}, (subject, options) => { ... })
//
//
// -- This is a dual command --
// Cypress.Commands.add('dismiss', { prevSubject: 'optional'}, (subject, options) => { ... })
//
//
// -- This will overwrite an existing command --
// Cypress.Commands.overwrite('visit', (originalFn, url, options) => { ... })

// CLICK ADD INPUT
Cypress.Commands.add('clickAddInputBtn', () => {
  cy.get('[data-testid="add-input-modal:confirm"]').click();
  cy.get('[data-testid="add-input-modal:confirm"]').should('not.exist');
});

// CLICK ADD BACKUP BTN
Cypress.Commands.add('clickAddBackupBtn', () => {
  cy.get('[data-testid="add-output-modal:add-backup"]').click();
});

// CLICK ADD BACKUP BTN
Cypress.Commands.add('clickEditInputBtn', (inputNum) => {
  cy.get(
    `:nth-child(${inputNum + 1}) > [data-testid="edit-input-modal:open"]`
  ).click();
});

// CLICK ADD OUTPUT
Cypress.Commands.add('clickAddOutputBtn', () => {
  cy.get('[data-testid="add-output-modal:confirm"]').click();
  cy.get('[data-testid="add-output-modal:confirm"]').should('not.exist');
});

// ALL OTPUTS START
Cypress.Commands.add('allOutputStart', () => {
  cy.get("span:contains('Start All')").click();
  cy.get('[slot="confirm"]').click();

  cy.get('[slot="confirm"]').should('not.exist');
});

// ALL OTPUTS STOP
Cypress.Commands.add('allOutputStop', () => {
  cy.get("span:contains('Stop All')").click();
  cy.get('[slot="confirm"]').click();

  cy.get('[slot="confirm"]').should('not.exist');
});

// DUMP STATE
Cypress.Commands.add('dumpState', () => {
  cy.get('.export-import-all').click();
  cy.get('.uk-textarea').then(($input) => {
    cy.writeFile('savedState.json', $input.val());
  });
  cy.get('.uk-modal-dialog .uk-close').eq(0).click();
});

// RESTORE STATE
Cypress.Commands.add('restoreState', () => {
  cy.get('.export-import-all').click();
  cy.readFile('savedState.json').then(($text) => {
    cy.get('.uk-textarea')
      .invoke('val', JSON.stringify($text))
      .trigger('input');
  });
  cy.get("button:contains('Replace')").click();
  cy.get("button:contains('Replace')").should('not.exist');
});

// REMOVE ALL INPUTS
Cypress.Commands.add('deleteAllInputs', () => {
  cy.get('.export-import-all').click();
  cy.get('.uk-textarea')
    .invoke(
      'val',
      `{
      "restreams":[],
      "version": "v1"
    }`
    )
    .trigger('input');
  cy.get("button:contains('Replace')").click();
  cy.get("button:contains('Replace')").should('not.exist');
});

// IMPORT ALL INPUTS
Cypress.Commands.add('importJsonConf', (host) => {
  cy.get('.export-import-all').click();
  cy.get('.uk-textarea')
    .invoke(
      'val',
      `{
  "version": "v1",
  "settings": {
    "title": null,
    "delete_confirmation": true,
    "enable_confirmation": true
  },
  "restreams": [
    {
      "id": "a430f4f3-639a-4572-aba4-0ad8380b2aa9",
      "key": "it",
      "label": "BACKUP",
      "input": {
        "id": "9723960c-dcdb-499c-94d1-398fc95f06fe",
        "key": "playback",
        "endpoints": [
          {
            "kind": "rtmp"
          }
        ],
        "src": {
          "failover_inputs": [
            {
              "id": "7746a36d-5d99-4fbc-9c1d-6209f759d9cb",
              "key": "primary",
              "endpoints": [
                {
                  "kind": "rtmp"
                }
              ],
              "enabled": true
            },
            {
              "id": "431e9b47-5a7a-45c3-8e70-216e38ff9492",
              "key": "backup1",
              "endpoints": [
                {
                  "kind": "rtmp"
                }
              ],
              "enabled": true
            }
          ]
        },
        "enabled": true
      },
      "outputs": [
        {
          "id": "e9e57309-3cca-4872-8c39-bdc1945afe6d",
          "dst": "rtmp://${host}/ru/primary",
          "label": "[Manual Start] FB",
          "preview_url": "https://www.facebook.com/allatratvbulgaria/posts/348462730347939",
          "enabled": true
        },
        {
          "id": "30396de9-aed6-4db1-8af6-3f1c88ba06cd",
          "dst": "rtmp://a.rtmp.youtube.com/live2/rwhetk-2s44",
          "label": "[Manual Start] YT",
          "preview_url": "https://youtu.be/jKl7txehM78",
          "enabled": true
        },
        {
          "id": "3433de9d-5910-4ff2-a5fc-23bcc677c85b",
          "dst": "file:///record.flv",
          "label": "File Record",
          "enabled": true
        }
      ]
    },
    {
      "id": "7cea855f-c250-4378-9d49-ccc93d22d3d1",
      "key": "ko",
      "label": "PULL",
      "input": {
        "id": "7abd81c1-e554-4541-87b4-70becfcff79a",
        "key": "primary",
        "endpoints": [
          {
            "kind": "rtmp"
          }
        ],
        "src": {
          "remote_url": "rtmp://${host}/en/primary"
        },
        "enabled": true
      }
    },
    {
      "id": "5914027b-b302-475b-ae21-5c99a3d5dddc",
      "key": "en",
      "label": "SINGLE",
      "input": {
        "id": "8383149e-bcea-4a3f-a8a9-661d6e72cbae",
        "key": "primary",
        "endpoints": [
          {
            "kind": "rtmp"
          }
        ],
        "enabled": true
      },
      "outputs": [
        {
          "id": "1c8229ee-c736-4d55-b5f0-d72c34ab5dea",
          "dst": "rtmp://${host}/it/backup1",
          "label": "Teamspeak",
          "preview_url": "https://www.youtube.com/watch?v=99567P5FiD0",
          "mixins": [
            {
              "src": "ts://ts.sameteem.com:9987/AFK\\\\/Muted",
              "delay": "3s 500ms",
              "sidechain": true
            },
            {
              "src": "ts://ts.sameteem.com:9987/AFK\\\\/Muted?name=MusicTest",
              "delay": "3s 500ms"
            },
            {
              "src": "ts://ts.sameteem.com:9987/AFK\\\\/Muted?name=MusicTest2",
              "delay": "3s 500ms"
            }
          ],
          "enabled": true
        },
        {
          "id": "06f08ac0-5d96-41d3-8782-14d06d403532",
          "dst": "rtmp://${host}/it/primary",
          "label": "Twitter",
          "preview_url": "https://creativesociety.com/ru",
          "enabled": true
        }
      ]
    },
    {
      "id": "399aefb2-e61e-46cf-a2fb-648bf252f4e6",
      "key": "ru",
      "label": "RU",
      "input": {
        "id": "6f5326c0-460f-4d87-958b-e21525f9c01e",
        "key": "primary",
        "endpoints": [
          {
            "kind": "rtmp"
          }
        ],
        "enabled": true
      }
    }
  ]
}`
    )
    .trigger('input');
  cy.get("button:contains('Replace')").click();
  cy.get("button:contains('Replace')").should('not.exist');
});

// IMPORT ALL INPUTS
Cypress.Commands.add('runTestStream', (rtmp) => {
  cy.exec(
    `ffmpeg -stream_loop -1 -re -nostdin -i ./cypress/data/test_video.mp4 -vcodec libx264 -preset:v ultrafast -acodec aac -f flv ${rtmp} >/dev/null 2>&1 &`
  );
});

const redColor = 'rgb(240, 80, 110)';
const greenColor = 'rgb(50, 210, 150)';
const brownColor = 'rgb(122, 81, 40)';

Cypress.Commands.add('checkStartedAllStated', () => {
  cy.get(
    '[data-icon="circle"][title="Serves failover live RTMP stream"]'
  ).should('have.css', 'color', greenColor);

  cy.get(
    '[data-icon="arrow-right"][title="Accepts primary live RTMP stream"]'
  ).should('have.css', 'color', greenColor);

  cy.get(
    '[data-icon="arrow-right"][title="Accepts backup1 live RTMP stream"]'
  ).should('have.css', 'color', greenColor);

  cy.get(
    '[data-icon="arrow-down"][title="Pulls primary live RTMP stream"]'
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

Cypress.Commands.add('checkStoppedAllStated', () => {
  cy.get(
    '[data-icon="dot-circle"][title="Serves failover live RTMP stream"]'
  ).should('have.css', 'color', redColor);

  cy.get(
    '[data-icon="arrow-right"][title="Accepts primary live RTMP stream"]'
  ).should('have.css', 'color', redColor);

  cy.get(
    '[data-icon="arrow-right"][title="Accepts backup1 live RTMP stream"]'
  ).should('have.css', 'color', redColor);

  cy.get(
    '[data-icon="arrow-down"][title="Pulls primary live RTMP stream"]'
  ).should('have.css', 'color', greenColor);

  cy.get('[data-testid=SINGLE] [data-icon="arrow-right"]').should(
    'have.css',
    'color',
    greenColor
  );

  cy.get('[data-testid=RU] [data-icon="arrow-right"]').should(
    'have.css',
    'color',
    redColor
  );

  cy.wait(5000);
  cy.get('[data-testid=Teamspeak] [data-icon="dot-circle"]').should(
    'have.css',
    'color',
    redColor
  );

  cy.get('[data-testid=Twitter] [data-icon="dot-circle"]').should(
    'have.css',
    'color',
    redColor
  );

  cy.get('[data-testid="[Manual Start] FB"] [data-icon="dot-circle"]').should(
    'have.css',
    'color',
    redColor
  );

  cy.get('[data-testid="[Manual Start] YT"] [data-icon="dot-circle"]').should(
    'have.css',
    'color',
    redColor
  );

  cy.get('[data-testid="File Record"] [data-icon="dot-circle"]').should(
    'have.css',
    'color',
    redColor
  );
});
