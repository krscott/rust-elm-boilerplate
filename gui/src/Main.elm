port module Main exposing (main)

import Browser
import Html
    exposing
        ( Html
        , button
        , div
        , input
        , text
        , textarea
        )
import Html.Attributes
    exposing
        ( autofocus
        , class
        , id
        , type_
        , value
        )
import Html.Events exposing (onClick, onInput)
import Json.Decode as Decode exposing (Decoder, field)
import Json.Encode exposing (Value, object, string)


port toRust : Value -> Cmd msg


port fromRust : (Value -> msg) -> Sub msg


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }



-- MODEL


type Model
    = Loading
    | EditFile String String


type Msg
    = ChangeState Model
    | SendToRust RustCommand


type RustCommand
    = Log String String
    | UploadFile String String


init : () -> ( Model, Cmd Msg )
init _ =
    ( Loading, Cmd.none )



----- UPDATE


toRustCmd : RustCommand -> Cmd Msg
toRustCmd command =
    let
        value =
            case command of
                Log level msg ->
                    object
                        [ ( "cmd", string "Log" )
                        , ( "level", string level )
                        , ( "msg", string msg )
                        ]

                UploadFile filename content ->
                    object
                        [ ( "cmd", string "UploadFile" )
                        , ( "filename", string filename )
                        , ( "content", string content )
                        ]
    in
    toRust value


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        SendToRust command ->
            ( model, toRustCmd command )

        ChangeState newModel ->
            ( newModel, Cmd.none )



-- VIEW


view : Model -> Html Msg
view model =
    div [ class "container" ]
        [ case model of
            Loading ->
                div [ class "centered" ] [ text "Loading..." ]

            EditFile filename content ->
                div [ id "edit-file-container" ]
                    [ div []
                        [ input
                            [ id "input-filename"
                            , type_ "text"
                            , value filename
                            , onInput (\newFilename -> ChangeState (EditFile newFilename content))
                            ]
                            []
                        ]
                    , div []
                        [ textarea
                            [ id "input-content"
                            , value content
                            , autofocus True
                            , onInput (\newContent -> ChangeState (EditFile filename newContent))
                            ]
                            []
                        ]
                    , div []
                        [ button
                            [ onClick (SendToRust (UploadFile filename content)) ]
                            [ text "Upload" ]
                        ]
                    ]
        ]



-- SUBSCRIPTIONS


modelEditFileDecoder : Decoder Model
modelEditFileDecoder =
    Decode.map2 EditFile
        (field "filename" Decode.string)
        (field "content" Decode.string)


chooseStateDecoder : String -> Decoder Model
chooseStateDecoder state =
    case state of
        "Loading" ->
            Decode.succeed Loading

        "EditFile" ->
            modelEditFileDecoder

        _ ->
            Decode.fail ("Invalid state type: " ++ state)


modelDecoder : Decoder Model
modelDecoder =
    field "state" Decode.string
        |> Decode.andThen chooseStateDecoder


decodeValue : Value -> Msg
decodeValue x =
    let
        result =
            Decode.decodeValue modelDecoder x
    in
    case result of
        Ok model ->
            ChangeState model

        Err err ->
            SendToRust (Log "Error" (Decode.errorToString err))


subscriptions : Model -> Sub Msg
subscriptions _ =
    fromRust decodeValue
