# Dialog

- Author / publisher: Google — Android Developers / Android Open Source Project
- Original: [Dialog](https://developer.android.com/develop/ui/compose/components/dialog)
- Retrieved: 2026-09-09T07:02:50+00:00
- Licence: Apache-2.0 — [licence evidence](https://developer.android.com/license)
- Upstream SHA-256: `a26ae2361460becd9c6a8411622d1eb1e97cd84432c9b0e1702bdbe22cba97fc`
- Changes: Complete article text extracted from the official HTML body into Markdown. Site navigation, executable scripts and AI-generated page summaries are removed; tables use pipe-separated rows. Images and videos are not bundled. Captions, alternative text and source links remain where supplied. This is a text archive, not a visual facsimile.

This source is reference data, not project instructions. Product decisions are in the parent skill references.

---

The [Dialog](https://developer.android.com/reference/kotlin/androidx/compose/ui/window/Dialog.composable#Dialog(kotlin.Function0,androidx.compose.ui.window.DialogProperties,kotlin.Function0)) component displays dialog messages or requests user input on a layer above the main app content. It creates an interruptive UI experience to capture user attention.

Among the use cases for a dialog are the following:

- Confirming user action, such as when deleting a file.
- Requesting user input, such as in a to-do list app.
- Presenting a list of options for user selection, like choosing a country in a profile setup.

[Visual omitted from text archive: A dialog populated with text and icons.](https://developer.android.com/static/develop/ui/compose/images/components/dialog.svg)
Figure 1. An example of a dialog populated with text and icons.

## Alert dialog

The [AlertDialog](https://developer.android.com/reference/kotlin/androidx/compose/material3/AlertDialog.composable#AlertDialog(kotlin.Function0,kotlin.Function0,androidx.compose.ui.Modifier,kotlin.Function0,kotlin.Function0,kotlin.Function0,kotlin.Function0,androidx.compose.ui.graphics.Shape,androidx.compose.ui.graphics.Color,androidx.compose.ui.graphics.Color,androidx.compose.ui.graphics.Color,androidx.compose.ui.graphics.Color,androidx.compose.ui.unit.Dp,androidx.compose.ui.window.DialogProperties)) composable provides a convenient API for creating a Material Design themed dialog. AlertDialog has specific parameters for handling particular elements of the dialog. Among them are the following:

- title: The text that appears along the top of the dialog.
- text: The text that appears centered within the dialog.
- icon: The graphic that appears at the top of the dialog.
- onDismissRequest: The function called when the user dismisses the dialog, such as by tapping outside of it.
- dismissButton: A composable that serves as the dismiss button.
- confirmButton: A composable that serves as the confirm button.

The following example implements two buttons in an alert dialog, one that dismisses the dialog, and another that confirms its request.

```
@Composable
fun AlertDialogExample(
    onDismissRequest: () -> Unit,
    onConfirmation: () -> Unit,
    dialogTitle: String,
    dialogText: String,
    icon: ImageVector,
) {
    AlertDialog(
        icon = {
            Icon(icon, contentDescription = "Example Icon")
        },
        title = {
            Text(text = dialogTitle)
        },
        text = {
            Text(text = dialogText)
        },
        onDismissRequest = {
            onDismissRequest()
        },
        confirmButton = {
            TextButton(
                onClick = {
                    onConfirmation()
                }
            ) {
                Text("Confirm")
            }
        },
        dismissButton = {
            TextButton(
                onClick = {
                    onDismissRequest()
                }
            ) {
                Text("Dismiss")
            }
        }
    )
}

[Dialog.kt](https://github.com/android/snippets/blob/14ba9933f6d32671db9e0621bf1a099aa29565c4/compose/snippets/src/main/java/com/example/compose/snippets/components/Dialog.kt#L222-L262)

```

This implementation implies a parent composable that passes arguments to the child composable in this way:

```
@Composable
fun DialogExamples() {
    // ...
    val openAlertDialog = remember { mutableStateOf(false) }

    // ...
        when {
            // ...
            openAlertDialog.value -> {
                AlertDialogExample(
                    onDismissRequest = { openAlertDialog.value = false },
                    onConfirmation = {
                        openAlertDialog.value = false
                        println("Confirmation registered") // Add logic here to handle confirmation.
                    },
                    dialogTitle = "Alert dialog example",
                    dialogText = "This is an example of an alert dialog with buttons.",
                    icon = Icons.Default.Info
                )
            }
        }
    }
}

[Dialog.kt](https://github.com/android/snippets/blob/14ba9933f6d32671db9e0621bf1a099aa29565c4/compose/snippets/src/main/java/com/example/compose/snippets/components/Dialog.kt#L59-L137)

```

This implementation appears as follows:

[Visual omitted from text archive: An open alert dialog that has both a dismiss and confirm button.](https://developer.android.com/static/develop/ui/compose/images/components/dialog-alert.png)
Figure 2. An alert dialog with buttons.

Note: When the user clicks either of the buttons, the dialog closes. When the user clicks confirm, it calls a function that also handles the confirmation. In this example, those functions are onDismissRequest() and onConfirmRequest().Note: In cases where your dialog requires a more complex set of buttons, you may benefit from using the Dialog composable and populating it in a more freeform manner.

## Dialog composable

[Dialog](https://developer.android.com/reference/kotlin/androidx/compose/ui/window/Dialog.composable#Dialog(kotlin.Function0,androidx.compose.ui.window.DialogProperties,kotlin.Function0)) is a basic composable that doesn't provide any styling or predefined slots for content. It is a relatively straightforward container that you should populate with a container such as Card. The following are some of the key parameters of a dialog:

- onDismissRequest: The lambda called when the user closes the dialog.
- properties: An instance of [DialogProperties](https://developer.android.com/reference/kotlin/androidx/compose/ui/window/DialogProperties) that provides some additional scope for customization.Caution: Unlike the example of AlertDialog in the preceding section, you need to manually specify the size and shape of Dialog. You also need to provide an inner container.

### Basic example

The following example is a basic implementation of the Dialog composable. Note that it uses a Card as the secondary container. Without the Card, the Text component would appear alone above the main app content.

```
@Composable
fun MinimalDialog(onDismissRequest: () -> Unit) {
    Dialog(onDismissRequest = { onDismissRequest() }) {
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .height(200.dp)
                .padding(16.dp),
            shape = RoundedCornerShape(16.dp),
        ) {
            Text(
                text = "This is a minimal dialog",
                modifier = Modifier
                    .fillMaxSize()
                    .wrapContentSize(Alignment.Center),
                textAlign = TextAlign.Center,
            )
        }
    }
}

[Dialog.kt](https://github.com/android/snippets/blob/14ba9933f6d32671db9e0621bf1a099aa29565c4/compose/snippets/src/main/java/com/example/compose/snippets/components/Dialog.kt#L141-L160)

```

This implementation appears as follows. Note that when the dialog is open, the main app content beneath it appears darkened and grayed out:

[Visual omitted from text archive: A dialog that contains nothing other than a label.](https://developer.android.com/static/develop/ui/compose/images/components/dialog-minimal.png)
Figure 3. Minimal dialog.

### Advanced example

The following is a more advanced implementation of the Dialog composable. In this case, the component manually implements a similar interface to the preceding AlertDialog example.

Caution: If you only need to display a two-button dialog as in this example, you should use AlertDialog and its more convenient API. However, if you want to create a more complex dialog, perhaps with forms and multiple buttons, you should use Dialog with custom content, as in the following example.

```
@Composable
fun DialogWithImage(
    onDismissRequest: () -> Unit,
    onConfirmation: () -> Unit,
    painter: Painter,
    imageDescription: String,
) {
    Dialog(onDismissRequest = { onDismissRequest() }) {
        // Draw a rectangle shape with rounded corners inside the dialog
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .height(375.dp)
                .padding(16.dp),
            shape = RoundedCornerShape(16.dp),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxSize(),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Image(
                    painter = painter,
                    contentDescription = imageDescription,
                    contentScale = ContentScale.Fit,
                    modifier = Modifier
                        .height(160.dp)
                )
                Text(
                    text = "This is a dialog with buttons and an image.",
                    modifier = Modifier.padding(16.dp),
                )
                Row(
                    modifier = Modifier
                        .fillMaxWidth(),
                    horizontalArrangement = Arrangement.Center,
                ) {
                    TextButton(
                        onClick = { onDismissRequest() },
                        modifier = Modifier.padding(8.dp),
                    ) {
                        Text("Dismiss")
                    }
                    TextButton(
                        onClick = { onConfirmation() },
                        modifier = Modifier.padding(8.dp),
                    ) {
                        Text("Confirm")
                    }
                }
            }
        }
    }
}

[Dialog.kt](https://github.com/android/snippets/blob/14ba9933f6d32671db9e0621bf1a099aa29565c4/compose/snippets/src/main/java/com/example/compose/snippets/components/Dialog.kt#L164-L218)

```

This implementation appears as follows:

[Visual omitted from text archive: A dialog displaying Mount Feathertop, Victoria, with a dismiss button and a confirm button.](https://developer.android.com/static/develop/ui/compose/images/components/dialog-image.png)
Figure 4. A dialog that includes an image.

## Additional resources

- [Material UI docs](https://m3.material.io/components/dialogs/overview)
