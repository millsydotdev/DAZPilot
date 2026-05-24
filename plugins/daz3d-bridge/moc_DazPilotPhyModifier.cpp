/****************************************************************************
** Meta object code from reading C++ file 'DazPilotPhyModifier.h'
**
** Created: Sun 24. May 19:26:02 2026
**      by: The Qt Meta Object Compiler version 63 (Qt 4.8.1)
**
** WARNING! All changes made in this file will be lost!
*****************************************************************************/

#include "DazPilotPhyModifier.h"
#if !defined(Q_MOC_OUTPUT_REVISION)
#error "The header file 'DazPilotPhyModifier.h' doesn't include <QObject>."
#elif Q_MOC_OUTPUT_REVISION != 63
#error "This file was generated using the moc from 4.8.1. It"
#error "cannot be used with the include files from this version of Qt."
#error "(The moc has changed too much.)"
#endif

QT_BEGIN_MOC_NAMESPACE
static const uint qt_meta_data_DazPilotPhyModifier[] = {

 // content:
       6,       // revision
       0,       // classname
       0,    0, // classinfo
       2,   14, // methods
       0,    0, // properties
       0,    0, // enums/sets
       0,    0, // constructors
       0,       // flags
       2,       // signalCount

 // signals: signature, parameters, type, tag, flags
      21,   20,   20,   20, 0x05,
      37,   20,   20,   20, 0x05,

       0        // eod
};

static const char qt_meta_stringdata_DazPilotPhyModifier[] = {
    "DazPilotPhyModifier\0\0assetModified()\0"
    "assetWasSaved()\0"
};

void DazPilotPhyModifier::qt_static_metacall(QObject *_o, QMetaObject::Call _c, int _id, void **_a)
{
    if (_c == QMetaObject::InvokeMetaMethod) {
        Q_ASSERT(staticMetaObject.cast(_o));
        DazPilotPhyModifier *_t = static_cast<DazPilotPhyModifier *>(_o);
        switch (_id) {
        case 0: _t->assetModified(); break;
        case 1: _t->assetWasSaved(); break;
        default: ;
        }
    }
    Q_UNUSED(_a);
}

const QMetaObjectExtraData DazPilotPhyModifier::staticMetaObjectExtraData = {
    0,  qt_static_metacall 
};

const QMetaObject DazPilotPhyModifier::staticMetaObject = {
    { &DzModifier::staticMetaObject, qt_meta_stringdata_DazPilotPhyModifier,
      qt_meta_data_DazPilotPhyModifier, &staticMetaObjectExtraData }
};

#ifdef Q_NO_DATA_RELOCATION
const QMetaObject &DazPilotPhyModifier::getStaticMetaObject() { return staticMetaObject; }
#endif //Q_NO_DATA_RELOCATION

const QMetaObject *DazPilotPhyModifier::metaObject() const
{
    return QObject::d_ptr->metaObject ? QObject::d_ptr->metaObject : &staticMetaObject;
}

void *DazPilotPhyModifier::qt_metacast(const char *_clname)
{
    if (!_clname) return 0;
    if (!strcmp(_clname, qt_meta_stringdata_DazPilotPhyModifier))
        return static_cast<void*>(const_cast< DazPilotPhyModifier*>(this));
    if (!strcmp(_clname, "DzSceneAsset"))
        return static_cast< DzSceneAsset*>(const_cast< DazPilotPhyModifier*>(this));
    if (!strcmp(_clname, "IDzSceneAsset"))
        return static_cast< IDzSceneAsset*>(const_cast< DazPilotPhyModifier*>(this));
    return DzModifier::qt_metacast(_clname);
}

int DazPilotPhyModifier::qt_metacall(QMetaObject::Call _c, int _id, void **_a)
{
    _id = DzModifier::qt_metacall(_c, _id, _a);
    if (_id < 0)
        return _id;
    if (_c == QMetaObject::InvokeMetaMethod) {
        if (_id < 2)
            qt_static_metacall(this, _c, _id, _a);
        _id -= 2;
    }
    return _id;
}

// SIGNAL 0
void DazPilotPhyModifier::assetModified()
{
    QMetaObject::activate(this, &staticMetaObject, 0, 0);
}

// SIGNAL 1
void DazPilotPhyModifier::assetWasSaved()
{
    QMetaObject::activate(this, &staticMetaObject, 1, 0);
}
static const uint qt_meta_data_DazPilotPhyModifierIO[] = {

 // content:
       6,       // revision
       0,       // classname
       0,    0, // classinfo
       0,    0, // methods
       0,    0, // properties
       0,    0, // enums/sets
       0,    0, // constructors
       0,       // flags
       0,       // signalCount

       0        // eod
};

static const char qt_meta_stringdata_DazPilotPhyModifierIO[] = {
    "DazPilotPhyModifierIO\0"
};

void DazPilotPhyModifierIO::qt_static_metacall(QObject *_o, QMetaObject::Call _c, int _id, void **_a)
{
    Q_UNUSED(_o);
    Q_UNUSED(_id);
    Q_UNUSED(_c);
    Q_UNUSED(_a);
}

const QMetaObjectExtraData DazPilotPhyModifierIO::staticMetaObjectExtraData = {
    0,  qt_static_metacall 
};

const QMetaObject DazPilotPhyModifierIO::staticMetaObject = {
    { &DzExtraModifierIO::staticMetaObject, qt_meta_stringdata_DazPilotPhyModifierIO,
      qt_meta_data_DazPilotPhyModifierIO, &staticMetaObjectExtraData }
};

#ifdef Q_NO_DATA_RELOCATION
const QMetaObject &DazPilotPhyModifierIO::getStaticMetaObject() { return staticMetaObject; }
#endif //Q_NO_DATA_RELOCATION

const QMetaObject *DazPilotPhyModifierIO::metaObject() const
{
    return QObject::d_ptr->metaObject ? QObject::d_ptr->metaObject : &staticMetaObject;
}

void *DazPilotPhyModifierIO::qt_metacast(const char *_clname)
{
    if (!_clname) return 0;
    if (!strcmp(_clname, qt_meta_stringdata_DazPilotPhyModifierIO))
        return static_cast<void*>(const_cast< DazPilotPhyModifierIO*>(this));
    return DzExtraModifierIO::qt_metacast(_clname);
}

int DazPilotPhyModifierIO::qt_metacall(QMetaObject::Call _c, int _id, void **_a)
{
    _id = DzExtraModifierIO::qt_metacall(_c, _id, _a);
    if (_id < 0)
        return _id;
    return _id;
}
QT_END_MOC_NAMESPACE
